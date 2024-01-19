use std::io::Write;
use std::slice;
use std::time::Duration;
use serialport::{ClearBuffer, SerialPort};

static BROM_HANDSHAKE: &'static [u8] = &[0xa0, 0x0a, 0x50, 0x05];
pub struct BootROM {
    port: Box<dyn SerialPort>,
}

impl BootROM {
    pub fn new(port: Box<dyn SerialPort>) -> BootROM {
        BootROM {
            port
        }
    }

    pub fn into_serial_port(self) -> Box<dyn SerialPort> {
        self.port
    }

    pub fn handshake(&mut self) {
        let mut i = 0;
        let mut rx_char = 0;
        self.port.set_timeout(Duration::from_millis(5)).unwrap();
        while i < BROM_HANDSHAKE.len() {
            self.port.write(&BROM_HANDSHAKE[i .. i+1])
                .expect("failed to write to port.");
            if let Ok(len) = self.port.read(slice::from_mut(&mut rx_char)) {
                if len == 1 && BROM_HANDSHAKE[i] == !rx_char {
                    i += 1;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));
        self.port.clear(ClearBuffer::Input).unwrap();
    }

    fn echo(&mut self, buf: &[u8]) {
        let mut rx_buf: Vec<u8> =vec![0; buf.len()];
        self.port.set_timeout(Duration::from_millis(100)).unwrap();
        self.port.write(buf).expect("failed to write to port.");
        self.port.read(rx_buf.as_mut_slice()).unwrap();
        if buf != rx_buf {
            panic!("returned data isn't the same. Tx: {:?} Rx: {:?}", buf, rx_buf);
        }
    }

    fn read_be16(&mut self) -> u16 {
        let mut rx_buf: Vec<u8> = vec![0; 2];
        let len = self.port.read(rx_buf.as_mut_slice()).unwrap();
        if len != 2 {
            panic!("not enough data returned.")
        }
        u16::from_be_bytes(rx_buf.try_into().unwrap())
    }

    fn read_be32(&mut self) -> u32 {
        let mut rx_buf: Vec<u8> = vec![0; 4];
        let len = self.port.read(rx_buf.as_mut_slice()).unwrap();
        if len != 4 {
            panic!("not enough data returned.")
        }
        u32::from_be_bytes(rx_buf.try_into().unwrap())
    }

    pub fn get_hw_code(&mut self) -> u16 {
        self.echo(slice::from_ref(&0xfd));
        let code = self.read_be16();
        let ret = self.read_be16();
        if ret != 0 {
            panic!("status: {}", ret);
        }
        code
    }

    pub fn get_hw_dict(&mut self) -> (u16, u16, u16) {
        self.echo(slice::from_ref(&0xfc));
        let hw_sub_code = self.read_be16();
        let hw_ver = self.read_be16();
        let sw_ver = self.read_be16();
        let ret = self.read_be16();
        if ret != 0 {
            panic!("status: {}", ret);
        }
        (hw_sub_code, hw_ver, sw_ver)
    }

    pub fn get_target_config(&mut self) -> (bool, bool, bool) {
        self.echo(slice::from_ref(&0xd8));
        let target_config = self.read_be32();
        let ret = self.read_be16();
        if ret != 0 {
            panic!("status: {}", ret);
        }
        let secure_boot = target_config & 1 != 0;
        let serial_link_authorization = target_config & 2 != 0;
        let download_agent_authorization = target_config & 4 != 0;
        (secure_boot, serial_link_authorization, download_agent_authorization)
    }

    pub fn send_da(&mut self, da_addr: u32, sig_len: u32, da_buf: &[u8]) -> u16 {
        self.echo(slice::from_ref(&0xd7));
        self.echo(&u32::to_be_bytes(da_addr));
        self.echo(&u32::to_be_bytes(da_buf.len() as u32 - sig_len));
        self.echo(&u32::to_be_bytes(sig_len));

        let ret = self.read_be16();
        if ret != 0 {
            panic!("send_da cmd status: {}", ret);
        }

        self.port.write(da_buf).unwrap();
        while self.port.bytes_to_write().unwrap() > 0 {
            std::thread::sleep(Duration::from_millis(200));
        }
        let checksum = self.read_be16();
        let ret = self.read_be16();
        if ret != 0 {
            panic!("send_da cmd status: {}", ret);
        }
        checksum
    }

    pub fn jump_da(&mut self, da_addr: u32) {
        self.echo(slice::from_ref(&0xd5));
        self.echo(&u32::to_be_bytes(da_addr));
        let ret = self.read_be16();
        if ret != 0 {
            panic!("jump_da cmd status: {}", ret);
        }
    }

    pub fn jump_da64(&mut self, da_addr: u32) {
        self.echo(slice::from_ref(&0xde));
        self.echo(&u32::to_be_bytes(da_addr));

        // 1 for 64-bit
        self.echo(slice::from_ref(&1));
        let ret = self.read_be16();
        if ret != 0 {
            panic!("jump_da64 cmd status: {}", ret);
        }

        // A magic number checked before resetting CPU to aarch64
        self.echo(slice::from_ref(&100));
        let ret = self.read_be16();
        if ret != 0 {
            panic!("jump_da64 magic status: {}", ret);
        }
    }
}