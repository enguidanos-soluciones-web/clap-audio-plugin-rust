#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![cfg_attr(target_os = "macos", allow(unexpected_cfgs))]

pub mod clap {
    include!(concat!(env!("OUT_DIR"), "/clap.rs"));
}

mod channel;
mod descriptor;
mod dsp;
mod entry;
mod extensions;
mod factory;
mod gestures;
mod gui;
mod helper;
mod host_notifier;
mod parameters;
mod plugin;
mod processors;
mod state;
mod version;
