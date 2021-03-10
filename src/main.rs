/// Impl SNTP v4
/// https://tools.ietf.org/html/rfc4330
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct NTPMessage {
    header: u8, // LI, VN, Mode
    stratum: u8,
    poll_interval: u8,
    precision: i8,
    root_delay: i32,
    root_dispersion: u32,
    reference_identifier: u32,
    reference_timestamp: u64,
    originate_timestamp: u64,
    receive_timestamp: u64,
    transmit_timestamp: u64,
}

impl NTPMessage {
    fn default_client_message() -> Self {
        Self {
            header: 0b00_100_011,
            ..Default::default()
        }
    }
    fn encode(&self) -> Vec<u8> {
        let ptr = self as *const _ as *const u8;
        let buf = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<NTPMessage>()) };
        buf.to_vec()
    }
}

use chrono::{DateTime, Duration, TimeZone, Utc};

fn to_utc(timestamp: u64) -> DateTime<Utc> {
    let start = DateTime::parse_from_rfc3339("1900-01-01T00:00:00Z").unwrap();
    let start: DateTime<Utc> = Utc.from_utc_datetime(&start.naive_utc());
    let timestamp = convert_endian(timestamp as u32);
    start + Duration::seconds(timestamp as i64)
}

fn convert_endian(s: u32) -> u32 {
    let mut ret = 0u32;
    for i in 0..4 {
        ret <<= 8;
        let p = (s >> (8 * i)) & ((1 << 8) - 1);
        ret |= p;
    }
    ret
}

use anyhow::{Context, Result};
use std::net::UdpSocket;

const NTP_SERVER: &str = "132.163.97.6:123";
fn main() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").with_context(|| "failed to bind")?;
    println!("{:?}", socket);
    socket.set_read_timeout(None)?;
    let buf = NTPMessage::default_client_message().encode();
    println!("connect");
    socket.connect(NTP_SERVER)?;
    println!("send");
    socket.send(&buf).with_context(|| "failed to send")?;
    let mut buf = vec![0; 1024];
    println!("receive");
    socket.recv(&mut buf)?;
    let msg = unsafe { *(buf.as_ptr() as *const _ as *const NTPMessage) };
    println!("Time: {}", to_utc(msg.receive_timestamp));
    Ok(())
}
