extern crate pnet;

use std::net::{IpAddr, Ipv4Addr};

use pnet::util::checksum;
use pnet::packet::Packet;
use pnet::transport::{transport_channel, icmp_packet_iter};
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::icmp::echo_request::{MutableEchoRequestPacket, IcmpCodes};

fn main() {
    const CHANNEL_BUF_SIZE: usize = 4096;
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

    let (mut tx, mut rx) = match transport_channel(CHANNEL_BUF_SIZE, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!("error happened {}", e),
    };

    // tx.set_ttl(1);

    // create ping packet
    let mut packet_contents = vec![0u8; 48];
    let mut ping_packet = MutableEchoRequestPacket::owned(packet_contents).unwrap();
    ping_packet.set_icmp_type(IcmpTypes::EchoRequest);
    ping_packet.set_icmp_code(IcmpCodes::NoCode);
    ping_packet.set_identifier(0);
    ping_packet.set_sequence_number(0);
    ping_packet.set_payload(b"Hello rust");

    let cksm = checksum(ping_packet.packet(), 1);
    ping_packet.set_checksum(cksm);

    let dest_addr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let sent_size = tx.send_to(ping_packet, dest_addr).expect("failed to send the packet");

    println!("{} bytes has been sent", sent_size);

    let mut iter = icmp_packet_iter(&mut rx);

    loop {
        match iter.next() {
            Ok((packet, _)) => {
                println!("{:?}", packet);
            }
            Err(e) => {
                panic!("error happened!");
            }
        }
    }
}
