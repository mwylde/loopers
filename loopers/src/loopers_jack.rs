use crossbeam_channel::{bounded, Receiver, Sender};
use jack::jack_sys::{
    jack_nframes_t, jack_position_t, jack_set_timebase_callback, jack_transport_state_t,
};
use jack::{AudioOut, Port, ProcessScope};
use loopers_common::api::Command;
use loopers_common::api::FrameTime;
use loopers_common::gui_channel::GuiSender;
use loopers_common::midi::MidiEvent;
use loopers_common::music::MetricStructure;
use loopers_common::Host;
use loopers_engine::Engine;
use loopers_gui::Gui;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{io, thread};

enum ClientChange {
    AddPort(u32),
    RemovePort(u32, Port<AudioOut>, Port<AudioOut>),
    TransportStart,
    TransportStop,
    TransportPosition(FrameTime, MetricStructure),
    Shutdown,
}

enum ClientChangeResponse {
    PortAdded(u32, Port<AudioOut>, Port<AudioOut>),
}

pub struct JackHost<'a> {
    looper_ports: &'a mut HashMap<u32, [Port<AudioOut>; 2]>,
    ps: Option<&'a ProcessScope>,
    port_change_tx: Sender<ClientChange>,
    port_change_resp: Receiver<ClientChangeResponse>,
}

impl<'a> Host<'a> for JackHost<'a> {
    fn add_looper(&mut self, id: u32) -> Result<(), String> {
        if !self.looper_ports.contains_key(&id) {
            if let Err(e) = self.port_change_tx.try_send(ClientChange::AddPort(id)) {
                warn!("Failed to send port add request: {:?}", e);
            }
        }

        Ok(())
    }

    fn remove_looper(&mut self, id: u32) -> Result<(), String> {
        if let Some([l, r]) = self.looper_ports.remove(&id) {
            if let Err(e) = self
                .port_change_tx
                .try_send(ClientChange::RemovePort(id, l, r))
            {
                warn!("Failed to send port remove request: {:?}", e);
            }
        }

        Ok(())
    }

    fn output_for_looper<'b>(&'b mut self, id: u32) -> Option<[&'b mut [f32]; 2]>
    where
        'a: 'b,
    {
        if let Ok(ClientChangeResponse::PortAdded(id, l, r)) = self.port_change_resp.try_recv() {
            self.looper_ports.insert(id, [l, r]);
        }

        let ps = self.ps?;
        let [l, r] = self.looper_ports.get_mut(&id)?;
        Some([l.as_mut_slice(ps), r.as_mut_slice(ps)])
    }

    fn start_transport(&mut self) {
        if let Err(e) = self.port_change_tx.try_send(ClientChange::TransportStart) {
            warn!("Failed to send start transport request: {:?}", e);
        }
    }

    fn stop_transport(&mut self) {
        if let Err(e) = self.port_change_tx.try_send(ClientChange::TransportStop) {
            warn!("Failed to send stop transport request: {:?}", e);
        }
    }

    fn set_transport_position(&mut self, time: FrameTime, ms: MetricStructure) {
        if let Err(e) = self
            .port_change_tx
            .try_send(ClientChange::TransportPosition(time, ms))
        {
            warn!("Failed to send position transport request: {:?}", e);
        }
    }
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        debug!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        debug!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        debug!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, _: jack::Frames) -> jack::Control {
        jack::Control::Quit
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        info!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        info!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        info!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        debug!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        info!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        warn!("JACK: xrun occurred");
        jack::Control::Continue
    }
}

fn update_transport_position(
    pos: &mut jack::TransportPosition,
    time: FrameTime,
    ms: MetricStructure,
) {
    pos.set_frame(time.0 as u32);

    let mut bbt = pos.bbt().unwrap_or_default();

    let beat = ms.tempo.beat(time);
    let beats_per_bar = ms.time_signature.upper;

    if beats_per_bar != 0 && beat >= 0 {
        bbt.bar = (beat / beats_per_bar as i64) as usize + 1;
        bbt.beat = (beat % beats_per_bar as i64) as usize + 1;
    }

    bbt.with_ticks_per_beat(ms.tempo.samples_per_beat() as f64);
    bbt.tick = (time.0 - beat * ms.tempo.samples_per_beat() as i64) as usize;

    bbt.with_bpm(ms.tempo.bpm() as f64).with_timesig(
        ms.time_signature.upper as f32,
        ms.time_signature.lower as f32,
    );

    if let Err(e) = pos.set_bbt(Some(bbt)) {
        error!("Invalid metric structure: {}", e);
    }
}

#[no_mangle]
pub extern "C" fn timebase_callback(
    _state: jack_transport_state_t,
    _nframes: jack_nframes_t,
    pos: *mut jack_position_t,
    new_pos: std::ffi::c_int,
    arg: *mut std::ffi::c_void,
) {
    let mut engine = unsafe { &mut *(arg as *mut Arc<Mutex<Engine>>) }
        .lock()
        .unwrap();
    let pos = unsafe { &mut *pos.cast::<jack::TransportPosition>() };
    if new_pos != 0 {
        engine.time = pos.frame() as i64;
    }
    update_transport_position(pos, FrameTime(engine.time), engine.metric_structure);
}

fn transport_listener(
    transport: jack::Transport,
    stop: Arc<AtomicBool>,
    command_sender: Sender<Command>,
) {
    std::thread::spawn(move || {
        let mut state_last: Option<jack::TransportState> = None;
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let Ok(jack::TransportStatePosition { state, .. }) = transport.query() {
                if Some(state) != state_last {
                    let _ = match state {
                        jack::TransportState::Starting | jack::TransportState::Rolling => {
                            command_sender.send(Command::Start(false))
                        }
                        jack::TransportState::Stopped => command_sender.send(Command::Pause(false)),
                    };
                    state_last = Some(state);
                }
            }
        }
    });
}

pub fn jack_main(
    gui: Option<Gui>,
    gui_sender: GuiSender,
    command_to_engine_sender: Sender<Command>,
    command_to_engine_receiver: Receiver<Command>,
    beat_normal: Vec<f32>,
    beat_emphasis: Vec<f32>,
    restore: bool,
) {
    // Create client
    let (client, _status) = jack::Client::new("loopers", jack::ClientOptions::NO_START_SERVER)
        .expect("Jack server is not running");

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("in_l", jack::AudioIn).unwrap();
    let in_b = client.register_port("in_r", jack::AudioIn).unwrap();
    let mut out_a = client.register_port("main_out_l", jack::AudioOut).unwrap();
    let mut out_b = client.register_port("main_out_r", jack::AudioOut).unwrap();

    let mut met_out_a = client
        .register_port("metronome_out_l", jack::AudioOut)
        .unwrap();
    let mut met_out_b = client
        .register_port("metronome_out_r", jack::AudioOut)
        .unwrap();

    let midi_in = client
        .register_port("loopers_midi_in", jack::MidiIn)
        .unwrap();

    let mut looper_ports: HashMap<u32, [Port<AudioOut>; 2]> = HashMap::new();

    let (port_change_tx, port_change_rx) = bounded(10);
    let (port_change_resp_tx, port_change_resp_rx) = bounded(10);

    let mut host = JackHost {
        looper_ports: &mut looper_ports,
        ps: None,
        port_change_tx: port_change_tx.clone(),
        port_change_resp: port_change_resp_rx.clone(),
    };

    let engine = Arc::new(Mutex::new(Engine::new(
        &mut host,
        gui_sender,
        command_to_engine_receiver,
        beat_normal,
        beat_emphasis,
        restore,
        client.sample_rate(),
    )));

    let mut engine_clone = engine.clone();
    unsafe {
        jack_set_timebase_callback(
            client.raw(),
            0,
            Some(timebase_callback),
            &mut engine_clone as *mut _ as *mut c_void,
        );
    }
    let process_port_change = port_change_tx.clone();

    let process_callback =
        move |_client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_bufs = [in_a.as_slice(ps), in_b.as_slice(ps)];
            let out_l = out_a.as_mut_slice(ps);
            let out_r = out_b.as_mut_slice(ps);
            for b in &mut *out_l {
                *b = 0f32;
            }
            for b in &mut *out_r {
                *b = 0f32;
            }

            for l in looper_ports.values_mut() {
                for c in l {
                    for v in c.as_mut_slice(ps) {
                        *v = 0f32;
                    }
                }
            }

            let mut met_bufs = [met_out_a.as_mut_slice(ps), met_out_b.as_mut_slice(ps)];
            for buf in &mut met_bufs {
                for b in &mut **buf {
                    *b = 0f32
                }
            }

            let mut host = JackHost {
                looper_ports: &mut looper_ports,
                ps: Some(ps),
                port_change_tx: process_port_change.clone(),
                port_change_resp: port_change_resp_rx.clone(),
            };

            let midi_events: Vec<MidiEvent> = midi_in
                .iter(ps)
                .filter_map(|e| MidiEvent::from_bytes(e.bytes))
                .collect();

            engine.lock().unwrap().process(
                &mut host,
                in_bufs,
                out_l,
                out_r,
                met_bufs,
                ps.n_frames() as u64,
                &midi_events,
            );

            jack::Control::Continue
        };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    let transport = active_client.as_client().transport();
    let stop_transport_listener = Arc::new(AtomicBool::new(false));

    transport_listener(
        transport,
        stop_transport_listener.clone(),
        command_to_engine_sender,
    );

    thread::spawn(move || loop {
        match port_change_rx.recv() {
            Ok(ClientChange::AddPort(id)) => {
                let l = active_client
                    .as_client()
                    .register_port(&format!("loop{}_out_l", id), jack::AudioOut)
                    .map_err(|e| format!("could not create jack port: {:?}", e));
                let r = active_client
                    .as_client()
                    .register_port(&format!("loop{}_out_r", id), jack::AudioOut)
                    .map_err(|e| format!("could not create jack port: {:?}", e));

                match (l, r) {
                    (Ok(l), Ok(r)) => {
                        if port_change_resp_tx
                            .send(ClientChangeResponse::PortAdded(id, l, r))
                            .is_err()
                        {
                            break;
                        }
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error!("Failed to register port with jack: {:?}", e);
                    }
                }
            }
            Ok(ClientChange::RemovePort(id, l, r)) => {
                if let Err(e) = active_client
                    .as_client()
                    .unregister_port(l)
                    .and_then(|()| active_client.as_client().unregister_port(r))
                {
                    error!("Unable to remove jack outputs: {:?}", e);
                }
                info!("removed ports for looper {}", id);
            }
            Ok(ClientChange::TransportStart) => {
                if let Err(e) = active_client.as_client().transport().start() {
                    error!("Failed to start jack transport: {:?}", e);
                }
            }
            Ok(ClientChange::TransportStop) => {
                if let Err(e) = active_client.as_client().transport().stop() {
                    error!("Failed to stop jack transport: {:?}", e);
                }
            }
            Ok(ClientChange::TransportPosition(time, ms)) => {
                let transport = active_client.as_client().transport();
                let mut pos = match transport.query() {
                    Ok(jack::TransportStatePosition { pos, state: _ }) => pos,
                    Err(e) => {
                        error!("Failed to query transport position: {}", e);
                        break;
                    }
                };

                update_transport_position(&mut pos, time, ms);

                if let Err(e) = transport.reposition(&pos) {
                    error!("Failed to reposition transport: {}", e);
                }
            }
            Ok(ClientChange::Shutdown) | Err(_) => {
                stop_transport_listener.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }
        }
    });

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

    if port_change_tx.send(ClientChange::Shutdown).is_err() {
        warn!("Failed to shutdown worker thread");
    }

    std::process::exit(0);
}
