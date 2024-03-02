use std::io::Write;
use std::slice;
use std::time::Duration;
use serialport::{ClearBuffer, SerialPort};

static BL2_HANDSHAKE_REQ: &[u8] = "mudl".as_bytes();
static BL2_HANDSHAKE_RESP: &[u8] = "TF-A".as_bytes();

pub struct BL2 {
    port: Box<dyn SerialPort>,
}

impl BL2 {
    pub fn new(port: Box<dyn SerialPort>) -> BL2 {
        BL2 {
            port
        }
    }

    pub fn into_serial_port(self) -> Box<dyn SerialPort> {
        self.port
    }

    pub fn handshake(&mut self) {
        let mut i = 0;
        let mut rx_char = 0;
        self.port.set_timeout(Duration::from_millis(500)).unwrap();
        while i < 4 {
            self.port.write_all(&BL2_HANDSHAKE_REQ[i..i + 1])
                .expect("failed to write to port.");
            if let Ok(()) = self.port.read_exact(slice::from_mut(&mut rx_char)) {
                if BL2_HANDSHAKE_RESP[i] == rx_char {
                    i += 1;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));
        self.port.clear(ClearBuffer::Input).unwrap();
    }

    fn echo(&mut self, buf: &[u8]) {
        let mut rx_buf: Vec<u8> = vec![0; buf.len()];
        self.port.write_all(buf).expect("failed to write to port.");
        self.port.read_exact(rx_buf.as_mut_slice()).unwrap();
        if buf != rx_buf {
            panic!("returned data isn't the same. Tx: {:?} Rx: {:?}", buf, rx_buf);
        }
    }

    fn read_be16(&mut self) -> u16 {
        let mut rx_buf: Vec<u8> = vec![0; 2];
        self.port.read_exact(rx_buf.as_mut_slice()).unwrap();
        u16::from_be_bytes(rx_buf.try_into().unwrap())
    }

    fn read_be32(&mut self) -> u32 {
        let mut rx_buf: Vec<u8> = vec![0; 4];
        self.port.read_exact(rx_buf.as_mut_slice()).unwrap();
        u32::from_be_bytes(rx_buf.try_into().unwrap())
    }

    pub fn version(&mut self) -> u8 {
        self.echo(slice::from_ref(&1));
        let mut rx_char = 0;
        self.port.read_exact(slice::from_mut(&mut rx_char)).unwrap();
        rx_char
    }

    pub fn set_baudrate(&mut self, baudrate: u32) {
        self.echo(slice::from_ref(&2));
        self.echo(&u32::to_be_bytes(baudrate));
        self.port.set_baud_rate(baudrate).expect("failed to switch baud rate.");
    }

    fn fip_packet_checksum(fip: &[u8]) -> u16 {
        let mut p = 0;
        let mut csum: u32 = 0;
        while fip.len() - p > 1 {
            let val = u16::from_be_bytes(fip[p..p + 2].try_into().unwrap());
            csum += val as u32;
            p += 2;
        }

        if fip.len() != p {
            csum += (fip.last().unwrap().to_owned() as u32) << 8;
        }

        while csum >> 16 != 0 {
            csum = ((csum >> 16) & 0xffff) + (csum & 0xffff);
        }

        csum as u16
    }

    fn send_fip_packet(&mut self, idx: u32, fip: &[u8]) -> bool {
        self.echo(&u32::to_be_bytes(idx));
        self.echo(&u16::to_be_bytes(fip.len() as u16));
        let checksum = BL2::fip_packet_checksum(fip);
        self.echo(&u16::to_be_bytes(checksum));
        self.port.write_all(fip).expect("failed to send fip packet.");

        while self.port.bytes_to_write().unwrap() > 0 {
            std::thread::sleep(Duration::from_millis(50));
        }

        let expected_idx = self.read_be32();
        let real_csum = self.read_be16();
        if expected_idx != idx {
            println!("Incorrect packet index: {} != {}", idx, expected_idx);
            false
        } else if real_csum != checksum {
            println!("Incorrect checksum: {:#x} != {:#x}", real_csum, checksum);
            false
        } else {
            true
        }
    }

    pub fn send_fip(&mut self, fip: &[u8]) {
        self.port.set_timeout(Duration::from_secs(2)).unwrap();
        self.echo(slice::from_ref(&3));
        self.echo(&u32::to_be_bytes(fip.len() as u32));
        let mut idx: u32 = 0;
        let mut pkt_len = 128;

        let mut p: usize = 0;
        while fip.len() - p > pkt_len {
            if self.send_fip_packet(idx, &fip[p..p + pkt_len]) {
                idx += 1;
                p += pkt_len;
                if pkt_len < 32768 {
                    pkt_len *= 2;
                } else if pkt_len < 65536 - 1024 {
                    pkt_len += 1024;
                }
            }
        }

        while !self.send_fip_packet(idx, &fip[p..]) {}
    }

    pub fn go(&mut self) {
        self.echo(slice::from_ref(&4));
    }
}