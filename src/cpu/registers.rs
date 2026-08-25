use super::flags::StatusRegister;

pub struct Registers
{
    a: u16, // Accumulator
    x: u16, // X Index Register
    y: u16, // Y Index Register

    dp: u16, // Direct Page Register
    db: u8, // Data Bank Register
    pb: u8, // Program Bank Register

    pc: u16, // Program Counter
    sp: u16, // Stack Pointer
    p: StatusRegister, // Status Register
    e: bool, // Emulation Flag
}

impl Registers 
{
    // Inicialization
    
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            dp: 0,
            db: 0,
            pb: 0,
            pc: 0,
            sp: 0x01FF, // Stack Pointer initialized to 0x01FF
            p: StatusRegister::new(),
            e: true, // Start in emulation mode
        }
    }

    pub fn reset(&mut self) {
        self.e = true; // Reset to emulation mode
        self.p.reset(); // Reset status register
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.dp = 0;
        self.db = 0;
        self.pb = 0;
        self.pc = 0;
        self.sp = 0x01FF; // Reset stack pointer to 0x01FF
    }

    // Accumulator

    pub fn a(&self) -> u16 {
        self.a
    }

    pub fn set_a(&mut self, value: u16) {
        self.a = value;
    }

    pub fn a_low(&self) -> u8 {
        (self.a & 0xFF) as u8
    }

    pub fn set_a_low(&mut self, value: u8) {
        self.a = (self.a & 0xFF00) | (value as u16);
    }

    pub fn a_is_8bit(&self) -> bool {
        self.p.m
    }

    // Index Registers (X/Y)
    // X Index
    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn set_x(&mut self, value: u16) {
        self.x = value;
    }

    pub fn x_low(&self) -> u8 {
        (self.x & 0xFF) as u8 // Read only the low byte
    }

    pub fn set_x_low(&mut self, value: u8) {
        self.x = (self.x & 0xFF00) | (value as u16); // Write only the low byte, but preserve the high byte
    }
    // Y index
    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn set_y(&mut self, value: u16) {
        self.y = value;
    }

    pub fn y_low(&self) -> u8 {
        (self.y & 0xFF) as u8 // Read only the low byte
    }

    pub fn set_y_low(&mut self, value: u8) {
        self.y = (self.y & 0xFF00) | (value as u16); // Write only the low byte, but preserve the high byte
    }

    // General Index
    pub fn index_is_8bit(&self) -> bool {
        self.p.x
    }

    // Stack Pointer (S)

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    pub fn increment_sp(&mut self) {
        self.sp = self.sp.wrapping_add(1);
    }

    pub fn decrement_sp(&mut self) {
        self.sp = self.sp.wrapping_sub(1);
    }

    // Direct Page Register (D)

    pub fn dp(&self) -> u16 {
        self.dp
    }

    pub fn set_dp(&mut self, value: u16) {
        self.dp = value;
    }

    // Data Bank Register (DB)

    pub fn db(&self) -> u8 {
        self.db
    }

    pub fn set_db(&mut self, value: u8) {
        self.db = value;
    }

    // Program Bank Register (PB)

    pub fn pb(&self) -> u8 {
        self.pb
    }

    pub fn set_pb(&mut self, value: u8) {
        self.pb = value;
    }

    // Program Counter (PC)

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

    // Status Register (P)

    pub fn status(&self) -> &StatusRegister {
        &self.p
    }
    
    pub fn status_mut(&mut self) -> &mut StatusRegister {
        &mut self.p 
    }
    
    // Emulation Flag (E)

    pub fn emulation_mode(&self) -> bool {
        self.e
    }

    pub fn set_emulation_mode(&mut self, value: bool) {
        self.e = value;
    }

    // Register width / CPU mode helpers

    pub fn is_native_mode(&self) -> bool {
        !self.e
    }

    pub fn is_emulation_mode(&self) -> bool {
        self.e
    }

    // Address helpers

    pub fn program_address(&self, offset: u16) -> u32 {
        (self.pb as u32) << 16 | (offset as u32)
    }

    pub fn data_address(&self, offset: u16) -> u32 {
        (self.db as u32) << 16 | (offset as u32)
    }

    //pub fn direct_address(&self, offset: u16) -> u32 {
    //    offset as u32
    //}
}