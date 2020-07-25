//! Socket address related components.
//!
//! # Binary Format of Socket Address
//!
//! ```text
//!  0                   1                   2                   3
//!  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |0 0 0 0 0 0 0 0|    Family     |           Port                |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                                                               |
//! |                 Address (32 bits or 128 bits)                 |
//! |                                                               |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!
//! Family: IPv4=1, IPv6=2
//! ```

use super::{Transaction, MAGIC_COOKIE};
use std::net::{IpAddr, SocketAddr};
use std::net::{Ipv4Addr, Ipv6Addr};
use bytes::{BytesMut, Buf, BufMut};

/// 协议类型
const FAMILY_IPV4: u8 = 0x01;
const FAMILY_IPV6: u8 = 0x02;

/// 创建IPV4缓冲区
fn copy_v4(buffer: BytesMut) -> [u8; 4] {
    let mut addr = [0u8; 4];
    buffer.copy_to_slice(&mut addr);
    addr
}

/// 创建IPV6缓冲区
fn copy_v6(buffer: BytesMut) -> [u8; 16] {
    let mut addr = [0u8; 16];
    buffer.copy_to_slice(&mut addr);
    addr
}

/// 解码IPV4
#[rustfmt::skip]
fn parse_ipv4(ip: Ipv4Addr, xor_port: u16) -> SocketAddr {
    let mut octets = ip.octets();
    for (i, b) in octets.iter_mut().enumerate() { *b ^= (MAGIC_COOKIE >> (24 - i * 8)) as u8; }
    SocketAddr::new(IpAddr::V4(From::from(octets)), xor_port)
}

/// 解码IPV6
#[rustfmt::skip]
fn parse_ipv6(id: Transaction, ip: Ipv6Addr, xor_port: u16) -> SocketAddr {
    let mut octets = ip.octets();
    for (i, b) in octets.iter_mut().enumerate().take(4) { *b ^= (MAGIC_COOKIE >> (24 - i * 8)) as u8; }
    for (i, b) in octets.iter_mut().enumerate().take(16).skip(4) { *b ^= id[i - 4]; }
    SocketAddr::new(IpAddr::V6(From::from(octets)), xor_port)
}

/// 将本地Addr类型，
/// 转为Xor类型.
#[rustfmt::skip]
fn from(addr: SocketAddr, id: Transaction) -> SocketAddr {
    match (addr.ip(), addr.port() ^ (MAGIC_COOKIE >> 16) as u16) {
        (IpAddr::V4(ip), port) => parse_ipv4(ip, port),
        (IpAddr::V6(ip), port) => parse_ipv6(id, ip, port)
    }
}

/// 将缓冲区解码为Addr
#[rustfmt::skip]
pub fn decoder(buf: Vec<u8>, id: Transaction) -> SocketAddr {
    let buffer = BytesMut::from(&buf[..]);
    let family = buffer.get_u8();
    let port = buffer.get_u16();
    from(SocketAddr::new(match family {
        FAMILY_IPV4 => IpAddr::V4(copy_v4(buffer).into()),
        FAMILY_IPV6 => IpAddr::V6(copy_v6(buffer).into())
    }, port), id)
}

/// 将Addr编码为缓冲区
#[rustfmt::skip]
pub fn encoder(addr: SocketAddr, id: Transaction) -> BytesMut {
    let xor_addr = from(addr, id);
    let mut buffer = BytesMut::new();
    buffer.put_u8(if xor_addr.is_ipv4() { FAMILY_IPV4 } else { FAMILY_IPV6 });
    if let IpAddr::V4(ip) = xor_addr.ip() { buffer.put(&ip.octets()[..]); }
    if let IpAddr::V6(ip) = xor_addr.ip() { buffer.put(&ip.octets()[..]); }
    buffer.put_u16(addr.port());
    buffer
}
