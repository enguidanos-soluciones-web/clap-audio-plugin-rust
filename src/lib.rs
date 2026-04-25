#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

pub mod clap {
    include!(concat!(env!("OUT_DIR"), "/clap.rs"));
}

mod descriptor;
mod entry;
mod extensions;
mod factory;
mod gestures;
mod gui;
mod helper;
mod nam;
mod plugin;
mod processors;
mod version;
