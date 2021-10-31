use std::{io, mem};
use std::ptr::null;
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use coreaudio::audio_unit::render_callback::{self, data};
use coreaudio::audio_unit::{AudioUnit, Element, SampleFormat, Scope, StreamFormat};
use coreaudio::sys::*;
use crossbeam_channel::{bounded, Receiver};
use loopers_common::api::Command;
use loopers_common::gui_channel::GuiSender;
use loopers_common::Host;
use loopers_engine::Engine;
use loopers_gui::Gui;

const SAMPLE_RATE: f64 = 44100.0;
type S = f32;
const SAMPLE_FORMAT: SampleFormat = SampleFormat::F32;

pub struct CoreAudioHost {
}

impl<'a> Host<'a> for CoreAudioHost {
    fn add_looper(&mut self, _: u32) -> Result<(), String> {
        // nothing to do
        Ok(())
    }

    fn remove_looper(&mut self, _: u32) -> Result<(), String> {
        // nothing to do
        Ok(())
    }

    fn output_for_looper<'b>(&'b mut self, _: u32) -> Option<[&'b mut [f32]; 2]> where 'a: 'b {
        // no per-looper outputs on coreaudio
        None
    }
}

pub fn coreaudio_main(gui: Option<Gui>,
                      gui_sender: GuiSender,
                      gui_to_engine_receiver: Receiver<Command>,
                      beat_normal: Vec<f32>,
                      beat_emphasis: Vec<f32>,
                      restore: bool) -> Result<(), coreaudio::Error> {
    let mut input_audio_unit = audio_unit_from_device(default_input_device().unwrap(), true)?;
    let mut output_audio_unit = audio_unit_from_device(default_output_device().unwrap(), false)?;

    let format_flag = match SAMPLE_FORMAT {
        SampleFormat::F32 => LinearPcmFlags::IS_FLOAT,
        SampleFormat::I32 | SampleFormat::I16 | SampleFormat::I8 => LinearPcmFlags::IS_SIGNED_INTEGER,
    };

    // Using IS_NON_INTERLEAVED everywhere because data::Interleaved is commented out / not implemented
    let in_stream_format = StreamFormat {
        sample_rate: SAMPLE_RATE,
        sample_format: SAMPLE_FORMAT,
        flags: format_flag | LinearPcmFlags::IS_PACKED | LinearPcmFlags::IS_NON_INTERLEAVED,
        channels_per_frame: 1,
    };

    let out_stream_format = StreamFormat {
        sample_rate: SAMPLE_RATE,
        sample_format: SAMPLE_FORMAT,
        flags: format_flag | LinearPcmFlags::IS_PACKED | LinearPcmFlags::IS_NON_INTERLEAVED,
        // you can change this to 1
        channels_per_frame: 2,
    };

    debug!("input={:#?}", &in_stream_format);
    debug!("output={:#?}", &out_stream_format);
    debug!("input_asbd={:#?}", &in_stream_format.to_asbd());
    debug!("output_asbd={:#?}", &out_stream_format.to_asbd());

    let mut host = CoreAudioHost {};

    let mut engine = Engine::new(
        &mut host,
        gui_sender,
        gui_to_engine_receiver,
        beat_normal,
        beat_emphasis,
        restore,
        SAMPLE_RATE as usize,
    );

    let id = kAudioUnitProperty_StreamFormat;
    let asbd = in_stream_format.to_asbd();
    input_audio_unit.set_property(id, Scope::Output, Element::Input, Some(&asbd))?;

    let asbd = out_stream_format.to_asbd();
    output_audio_unit.set_property(id, Scope::Input, Element::Output, Some(&asbd))?;

    let (mut sender_l, receiver_l) = bounded(2048);
    let (mut sender_r, receiver_r) = bounded(2048);

    type Args = render_callback::Args<data::NonInterleaved<S>>;

    input_audio_unit.set_input_callback(move |args| {
        let Args {
            num_frames,
            mut data,
            ..
        } = args;
        for i in 0..num_frames {
            let queues = [&mut sender_l, &mut sender_r];
            for (ch, channel) in data.channels_mut().enumerate() {
                let value: S = channel[i];
                queues[ch].send(value).unwrap();
            }
        }
        Ok(())
    })?;
    input_audio_unit.start()?;

    let mut input_l = vec![0f32; 4096];
    let mut input_r = vec![0f32; 4096];
    let mut output_l = vec![0f32; 4096];
    let mut output_r = vec![0f32; 4096];
    let mut met_l = vec![0f32; 4096];
    let mut met_r = vec![0f32; 4096];

    output_audio_unit.set_render_callback(move |args: Args| {
        let Args {
            num_frames,
            mut data,
            ..
        } = args;

        if num_frames > output_l.len() {
            panic!("frame is too big: {}", num_frames);
        }

        for i in 0..num_frames {
            input_l[i] = receiver_l.try_recv().unwrap_or(0.0);
            input_r[i] = receiver_r.try_recv().unwrap_or(0.0);

            output_l[i] = 0.0;
            output_r[i] = 0.0;

            met_l[i] = 0.0;
            met_r[i] = 0.0;
        }

        engine.process(&mut host,
                       [&input_l[0..num_frames], &input_r[0..num_frames]],
                       &mut output_l[0..num_frames],
                       &mut output_r[0..num_frames],
                       [&mut met_l[0..num_frames], &mut met_r[0..num_frames]],
                       num_frames as u64,
                       &[][..]);

        let outputs = [&output_l, &output_r];
        let mets = [&met_l, &met_r];

        for i in 0..num_frames {
            for (ch, channel) in data.channels_mut().enumerate() {
                if ch < 2 {
                    channel[i] = outputs[ch][i] + mets[ch][i];
                }
            }
        }
        Ok(())
    })?;
    output_audio_unit.start()?;

    // start the gui
    if let Some(gui) = gui {
        gui.start();
    } else {
        loop {
            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input).ok();
            if user_input == "q" {
                break;
            }
        }
    }

    std::process::exit(0);
}

pub fn default_output_device() -> Option<AudioDeviceID> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let audio_device_id: AudioDeviceID = 0;
    let data_size = mem::size_of::<AudioDeviceID>();
    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_address as *const _,
            0,
            null(),
            &data_size as *const _ as *mut _,
            &audio_device_id as *const _ as *mut _,
        )
    };
    if status != kAudioHardwareNoError as i32 {
        return None;
    }

    Some(audio_device_id)
}

pub fn default_input_device() -> Option<AudioDeviceID> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };

    let audio_device_id: AudioDeviceID = 0;
    let data_size = mem::size_of::<AudioDeviceID>();
    let status = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_address as *const _,
            0,
            null(),
            &data_size as *const _ as *mut _,
            &audio_device_id as *const _ as *mut _,
        )
    };
    if status != kAudioHardwareNoError as i32 {
        return None;
    }

    Some(audio_device_id)
}

fn audio_unit_from_device(
    device_id: AudioDeviceID,
    input: bool,
) -> Result<AudioUnit, coreaudio::Error> {
    let mut audio_unit = AudioUnit::new(coreaudio::audio_unit::IOType::HalOutput)?;

    if input {
        // Enable input processing.
        let enable_input = 1u32;
        audio_unit.set_property(
            kAudioOutputUnitProperty_EnableIO,
            Scope::Input,
            Element::Input,
            Some(&enable_input),
        )?;

        // Disable output processing.
        let disable_output = 0u32;
        audio_unit.set_property(
            kAudioOutputUnitProperty_EnableIO,
            Scope::Output,
            Element::Output,
            Some(&disable_output),
        )?;
    }

    audio_unit.set_property(
        kAudioOutputUnitProperty_CurrentDevice,
        Scope::Global,
        Element::Output,
        Some(&device_id),
    )?;

    Ok(audio_unit)
}

