# Loopers

![master status](https://github.com/mwylde/loopers/workflows/Rust/badge.svg?branch=master)

Loopers is graphical [live looper](http://www.livelooping.org/), written in Rust, designed for ease of use and 
rock-solid stability. It can be used as a practice tool, compositional aid, or for performing looped works in a live 
setting. 

Currently it runs only on Linux as a standalone [Jack](https://jackaudio.org/) application, which allows it to interface
with other Jack applications like effect racks, software instruments, and DAWs.

**INSERT GIF**

The system is modeled as a series of hardware loop units (like the Boss Loop Station) which are synchronized with
a single time control. The number of loop units is limited only by your display size, and loop lengths are limited only
by available memory.

## Features

* Multiple loops synchronized by a common time control
* Loops can be recorded to (setting loop length), overdubbed, cleared, muted, and soloed
* Up to four parts can be used to divide up portions of a performance
* Supports beat, measure, and free quantization of loop commands making it easy to keep things in sync
* Every operation can be controlled via the GUI or via MIDI
* Sessions can be saved and restored
* A built-in metronome (on a separate Jack output) helps keep you in time with your loops
* No limitations on loop lengths aside from your computer's memory
* It's fun!
