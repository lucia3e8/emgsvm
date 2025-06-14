#[derive(Debug, Clone, Copy)]
pub enum Command {
    Null,
    Reset,
    Standby,
    Wakeup,
    Lock,
    Unlock,
    RReg {
        addr: u8,
        count: u8,
    },
    WReg {
        addr: u8,
        count: u8,
    },
}

impl Command {
    pub fn to_u16(self) -> u16 {
        match self {
            Command::Null => 0b0000_0000_0000_0000,
            Command::Reset => 0b0000_0000_0001_0001,
            Command::Standby => 0b0000_0000_0010_0010,
            Command::Wakeup => 0b0000_0000_0011_0011,
            Command::Lock => 0b0000_0101_0101_0101,
            Command::Unlock => 0b0000_0110_0101_0101,
            Command::RReg {
                addr,
                count
            } => {
                let prefix = 0b1010_0000_0000_0000;
                let a = ((addr & 0b11111) as u16) << 7;
                let n = (count & 0b1111111) as u16;
                prefix | a | n
            }

            Command::WReg { addr, count } => {
                let a = ((addr & 0b11111) as u16) << 7;
                let n = (count & 0b1111_1111) as u16;
                0b0100_0000_0000_0000 | a | n
            }
        }
    }

    pub fn encode(self, buffer: &mut [u8]) {
        let command = self.to_u16();
        buffer[0] = (command >> 8) as u8;
        buffer[1] = command as u8;
    }
}
