extern crate pnet;

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use pnet::util::checksum;
use pnet::packet::Packet;
use pnet::transport::{transport_channel, icmp_packet_iter};
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::icmp::echo_request::{MutableEchoRequestPacket, IcmpCodes};

fn create_ping_packet<'a>(sequence_number: u16) -> MutableEchoRequestPacket<'a> {
    // create ping packet
    let packet_contents = vec![0u8; 48];
    let mut ping_packet = MutableEchoRequestPacket::owned(packet_contents).unwrap();
    ping_packet.set_icmp_type(IcmpTypes::EchoRequest);
    ping_packet.set_icmp_code(IcmpCodes::NoCode);
    ping_packet.set_identifier(0);
    ping_packet.set_sequence_number(0);
    ping_packet.set_payload(b"Hello rust");

    let cksm = checksum(ping_packet.packet(), 1);
    ping_packet.set_checksum(cksm);

    ping_packet
}

fn main() {
    const CHANNEL_BUF_SIZE: usize = 4096;
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

    let (mut tx, mut rx) = match transport_channel(CHANNEL_BUF_SIZE, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!("error happened {}", e),
    };

    let dest_addr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let mut iter = icmp_packet_iter(&mut rx);

    let mut seq = 1;
    loop {
        // incrementally set the ttl.
        tx.set_ttl(seq).expect("failed to set the ttl");

        // create and send the ping packet.
        let ping_packet = create_ping_packet(seq as u16);
        tx.send_to(ping_packet, dest_addr).expect("failed to send the packet");

        // we may hanging there waiting for one hog without timeout.
        match iter.next_with_timeout(Duration::new(5, 0)) {
            Ok(result) => {
                match result {
                    Some((packet, addr)) => {
                        let icmp_type = packet.get_icmp_type();
                        match icmp_type {
                            // ttl exhaused, hog detected.
                            IcmpTypes::TimeExceeded => {
                                println!("{} {}", seq, addr);
                            }
                            // we've reached our target.
                            IcmpTypes::EchoReply => {
                                if addr == dest_addr {
                                    println!("{} {}", seq, addr);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    // timed out.
                    None => {
                        println!("{} * * *", seq);
                    }
                }
            }
            Err(e) => {
                panic!("error happened: {}", e);
            }
        }
        seq += 1;
    }
}
