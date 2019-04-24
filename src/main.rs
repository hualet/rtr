extern crate pnet;

use std::net::{IpAddr, Ipv4Addr};

use pnet::util::checksum;
use pnet::transport::{transport_channel, icmp_packet_iter};
use pnet::transport::TransportChannelType::Layer3;

use pnet::packet::Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::icmp::echo_request::{MutableEchoRequestPacket, IcmpCodes};

fn main() {
    const channel_buf_size: usize = 4096;
    let protocol = Layer3(IpNextHeaderProtocols::Icmp);

    let (mut tx, mut rx) = match transport_channel(channel_buf_size, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!("error happened {}", e),
    };

    // tx.set_ttl(1);

    let mut packet_buf = [0u8; 128];
    let mut payload_buf = [0u8; 10];

    let mut packet = MutableEchoRequestPacket::new(&mut packet_buf).expect("failed to new MutableEchoRequestPacket");
    packet.set_icmp_type(IcmpTypes::EchoRequest);
    packet.set_icmp_code(IcmpCodes::NoCode);
    packet.set_identifier(0);
    packet.set_sequence_number(0);
    packet.set_checksum(0);
    packet.set_payload(&mut payload_buf);

    let cksm = checksum(packet.packet(), 1);
    packet.set_checksum(cksm);

    println!("{}", cksm);

    let dest_addr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let sent_size = tx.send_to(packet, dest_addr).expect("failed to send the packet");

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
