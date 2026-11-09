use super::status::StatusRegister;

/// The 65816 programmer-visible registers.
pub struct Registers {
    a: u16, // Accumulator (C in native mode, A is the low byte)
    x: u16, // X index
    y: u16, // Y index

    dp: u16, // Direct page register (D)
    db: u8,  // Data bank register (DB)
    pb: u8,  // Program bank register (PB)

    pc: u16,           // Program counter
    sp: u16,           // Stack pointer (S)
    p: StatusRegister, // Status register (P)
    e: bool,           // Emulation flag (E)
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            dp: 0,
            db: 0,
            pb: 0,
            pc: 0,
            sp: 0x01FF,
            p: StatusRegister::new(),
            e: true, // The CPU starts in emulation mode
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // Accumulator

    pub fn a(&self) -> u16 {
        self.a
    }

    pub fn set_a(&mut self, value: u16) {
        self.a = value;
    }

    pub fn set_a_low(&mut self, value: u8) {
        self.a = (self.a & 0xFF00) | (value as u16);
    }

    pub fn a_is_8bit(&self) -> bool {
        self.p.m
    }

    // Index registers

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn set_x(&mut self, value: u16) {
        self.x = value;
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn set_y(&mut self, value: u16) {
        self.y = value;
    }

    pub fn index_is_8bit(&self) -> bool {
        self.p.x
    }

    /// X as an index: only the low byte counts when the index registers are 8-bit.
    pub fn index_x(&self) -> u16 {
        if self.index_is_8bit() { self.x & 0xFF } else { self.x }
    }

    /// Y as an index: only the low byte counts when the index registers are 8-bit.
    pub fn index_y(&self) -> u16 {
        if self.index_is_8bit() { self.y & 0xFF } else { self.y }
    }

    // Stack pointer

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    // Direct page

    pub fn dp(&self) -> u16 {
        self.dp
    }

    pub fn set_dp(&mut self, value: u16) {
        self.dp = value;
    }

    // Data bank

    pub fn db(&self) -> u8 {
        self.db
    }

    pub fn set_db(&mut self, value: u8) {
        self.db = value;
    }

    // Program bank and counter

    pub fn pb(&self) -> u8 {
        self.pb
    }

    pub fn set_pb(&mut self, value: u8) {
        self.pb = value;
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn increment_pc(&mut self, by: u16) {
        self.pc = self.pc.wrapping_add(by);
    }

    pub fn decrement_pc(&mut self, by: u16) {
        self.pc = self.pc.wrapping_sub(by);
    }

    // Status register

    pub fn status(&self) -> &StatusRegister {
        &self.p
    }

    pub fn status_mut(&mut self) -> &mut StatusRegister {
        &mut self.p
    }

    // Emulation flag

    pub fn emulation_mode(&self) -> bool {
        self.e
    }

    pub fn set_emulation_mode(&mut self, value: bool) {
        self.e = value;
    }

    pub fn is_native_mode(&self) -> bool {
        !self.e
    }

    pub fn is_emulation_mode(&self) -> bool {
        self.e
    }

    // Full 24-bit addresses: bank register plus a 16-bit offset

    pub fn program_address(&self, offset: u16) -> u32 {
        (self.pb as u32) << 16 | (offset as u32)
    }

    pub fn data_address(&self, offset: u16) -> u32 {
        (self.db as u32) << 16 | (offset as u32)
    }
}
