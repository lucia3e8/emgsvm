#[derive(Clone, Copy)]
pub struct Status {
    pub lock: bool,
    pub f_resync: bool,
    pub reg_map: bool,
    pub crc_err: bool,
    pub crc_type: bool,
    pub reset: bool,
    pub wlength: u8, // 2 bits
    pub drdy: u8,    // 8 bits: DRDY0..DRDY7 packed
}

impl Status {
    // only reads first 2 bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 2, "gimme at least 2 bytes, peasant");
        let word = u16::from_be_bytes([bytes[0], bytes[1]]);

        Self {
            lock: (word & (1 << 15)) != 0,
            f_resync: (word & (1 << 14)) != 0,
            reg_map: (word & (1 << 13)) != 0,
            crc_err: (word & (1 << 12)) != 0,
            crc_type: (word & (1 << 11)) != 0,
            reset: (word & (1 << 10)) != 0,
            wlength: ((word >> 8) & 0b11) as u8,
            drdy: (word & 0xFF) as u8,
        }
    }
}

impl core::fmt::Debug for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "lock={} ", self.lock as u8)?;
        write!(f, "f_resync={} ", self.f_resync as u8)?;
        write!(f, "reg_map={} ", self.reg_map as u8)?;
        write!(f, "crc_err={} ", self.crc_err as u8)?;
        write!(f, "crc_type={} ", self.crc_type as u8)?;
        write!(f, "reset={} ", self.reset as u8)?;
        write!(f, "wlength={:02b} ", self.wlength)?;

        write!(f, "drdy=0b{:08b} ", self.drdy)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mode {
    pub regcrc_en: bool,
    pub rx_crc_en: bool,
    pub crc_type: bool,
    pub reset: bool,
    pub wlength: u8,  // 2 bits
    pub drdy_sel: u8, // 2 bits
    pub drdy_hiz: bool,
    pub drdy_fmt: bool,
}

impl Mode {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 2);
        let word = u16::from_le_bytes([bytes[0], bytes[1]]);
        Self {
            regcrc_en: (word & (1 << 13)) != 0,
            rx_crc_en: (word & (1 << 12)) != 0,
            crc_type: (word & (1 << 11)) != 0,
            reset: (word & (1 << 10)) != 0,
            wlength: ((word >> 8) & 0b11) as u8,
            drdy_sel: ((word >> 6) & 0b11) as u8,
            drdy_hiz: (word & (1 << 5)) != 0,
            drdy_fmt: (word & (1 << 4)) != 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    pub ch_enable: u8, // bits 15..8 = CH7_EN..CH0_EN
    pub osr: u8,       // bits 5..3
    pub pwr: u8,       // bits 1..0
}

impl Clock {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 2);
        let word = u16::from_le_bytes([bytes[0], bytes[1]]);
        Self {
            ch_enable: (word >> 8) as u8,
            osr: ((word >> 3) & 0b111) as u8,
            pwr: (word & 0b11) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gain {
    pub pgagain_low: u8,  // bits 2:0
    pub pgagain_high: u8, // bits 10:8
}

impl Gain {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 2);
        let word = u16::from_le_bytes([bytes[0], bytes[1]]);
        Self {
            pgagain_low: (word & 0b111) as u8,
            pgagain_high: ((word >> 8) & 0b111) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cfg {
    pub cd_allch: bool,
    pub cd_num: u8, // bits 13:12
    pub cd_len: u8, // bits 11:10
    pub cd_en: bool,
    pub gc_dly: u8, // bits 7:4
    pub gc_en: bool,
}

impl Cfg {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() >= 2);
        let word = u16::from_le_bytes([bytes[0], bytes[1]]);
        Self {
            cd_allch: (word & (1 << 15)) != 0,
            cd_num: ((word >> 12) & 0b11) as u8,
            cd_len: ((word >> 10) & 0b11) as u8,
            cd_en: (word & (1 << 9)) != 0,
            gc_dly: ((word >> 4) & 0b1111) as u8,
            gc_en: (word & (1 << 0)) != 0,
        }
    }
}
