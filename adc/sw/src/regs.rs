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
    pub fn from_word(word: u16) -> Self {
        Self {
            lock:      (word & (1 << 15)) != 0,
            f_resync:  (word & (1 << 14)) != 0,
            reg_map:   (word & (1 << 13)) != 0,
            crc_err:   (word & (1 << 12)) != 0,
            crc_type:  (word & (1 << 11)) != 0,
            reset:     (word & (1 << 10)) != 0,
            wlength:   ((word >> 8) & 0b11) as u8,
            drdy:      (word & 0xFF) as u8,
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
