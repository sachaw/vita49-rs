// SPDX-FileCopyrightText: 2026 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use etherparse::{SlicedPacket, TransportSlice};
use pcap::Capture;
use std::env;
use vita49::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_file.pcap>", args[0]);
        std::process::exit(1);
    }
    let pcap_file = &args[1];

    let mut cap = match Capture::from_file(pcap_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to open PCAP file: {e}");
            std::process::exit(1);
        }
    };

    println!("Reading {pcap_file}...");
    while let Ok(packet) = cap.next_packet() {
        // Parse the raw packet bytes starting from the Ethernet layer
        match SlicedPacket::from_ethernet(packet.data) {
            Ok(sliced_packet) => {
                // Assume that the packet is sent via UDP
                if let Some(TransportSlice::Udp(udp_slice)) = sliced_packet.transport {
                    // Extract the payload directly from the UDP slice
                    let payload = udp_slice.payload();

                    // Try to parse it as a VRT packet
                    let packet = Vrt::try_from(payload).expect("failed to parse packet as VITA 49");

                    // Do different things depending on the type of packet
                    match packet.header().packet_type() {
                        // If it's a signal data packet, just print the payload length
                        PacketType::SignalData => {
                            println!(
                                "Signal data packet with stream ID 0x{:X} and a payload of length {}",
                                &packet.stream_id().unwrap(),
                                packet.payload().signal_data().unwrap().payload_size_bytes()
                            );
                        }
                        // If it's a context or command packet, print the fields (using the pre-
                        // implemented Display trait)
                        PacketType::Context => {
                            println!("Context packet:\n{}", packet.payload().context().unwrap());
                        }
                        PacketType::Command => {
                            println!("Command packet:\n{}", packet.payload().command().unwrap());
                        }
                        // Other packet types are not covered in this example
                        _ => unimplemented!(),
                    }
                }
            }
            Err(_) => {
                // Ignore packets that don't parse correctly (e.g., corrupted or non-Ethernet data-link layers)
                continue;
            }
        }
    }
}
