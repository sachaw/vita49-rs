// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
#![no_main]
#![doc = include_str!("../README.md")]

#[cxx::bridge]
mod ffi {
    pub struct MySignalData {
        stream_id: u32,
        signal_data: Vec<u8>,
    }

    extern "Rust" {
        fn parse_vita49(packet_data: &[u8]) -> MySignalData;
    }
}

use vita49::prelude::*;

use ffi::MySignalData;

pub fn parse_vita49(packet_data: &[u8]) -> MySignalData {
    match Vrt::try_from(packet_data) {
        Ok(packet) => match packet.header().packet_type() {
            PacketType::SignalData => {
                println!(
                    "[RUST] Parsed signal data packet with a {} byte payload",
                    packet.payload().signal_data().unwrap().payload_size_bytes()
                );
                MySignalData {
                    stream_id: packet.stream_id().unwrap(),
                    signal_data: packet.payload().signal_data().unwrap().payload().into(),
                }
            }
            // Other packet types are not covered in this example
            _ => unimplemented!(),
        },
        Err(e) => panic!("Failed to parse: {e}"),
    }
}
