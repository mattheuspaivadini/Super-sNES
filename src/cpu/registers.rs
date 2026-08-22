use super::flags::StatusRegister

pub struct Registers
{
    a: u16,
    x: u16,
    y: u16,

    pc: u16,
    sp: u16,
    p: StatusRegister,
    e: bool,
}

impl Registers 
{
    // Inicialization
    
    pub fn new() -> Self {}

    pub fn reset(&mut self) {}

    // Accumulator

    pub fn a(&self) -> u16 {}

    pub fn set_a(&mut self, value: u16) {}

    pub fn a_low(&self) -> u8 {}

    pub fn set_a_low(&mut self, value: u8) {}

    pub fn a_is_8bit(&self) -> bool {}

    // Index Registers (X/Y)
    // X Index
    pub fn x(&self) -> u16 {}

    pub fn set_x(&mut self, value: u16) {}
    // Y index
    pub fn y(&self) -> u16 {}
    
    pub fn set_y(&mut self, value: u16) {}
    // General Index
    pub fn index_is_8bit(&self) -> bool {}

    // Stack Pointer (S)

    pub fn sp(&self) -> u16 {}

    pub fn set_sp(&mut self, value: u16) {}

    pub fn increment_sp(&mut self) {}

    pub fn decrement_sp(&mut self) {}

    // Direct Page Register (D)

    pub fn dp(elf) -> u16 {}

    pub fn set_dp(&mut self, value: u16) {}

    // Data Bank Register (DB)

    pub fn db(&self) -> u8 {}

    pub fn set_db(&mut self, value: u8) {}

    // Program Bank Register (PB)

    pub fn pb(&self) -> u8 {}

    pub fn set_pb(&mut self, value: u8) {}

    // Program Counter (PC)

    pub fn pc(&self) -> u16 {}

    pub fn set_pc(&mut self, value: u16) {}

    pub fn increment_pc(&mut self) {}

    pub fn decrement_pc(&mut self) {}

    // Status Register (P)

    pub fn status(&self) -> &StatusRegister {}
    
    pub fn status_mut(&mut self) -> &mut StatusRegister {}
    
    // Emulation Flag (E)

    pub fn emulation_mode(&self) -> bool {}

    pub fn set_emulation_mode(&mut self, value: bool) {}

    // Register width / CPU mode helpers

    pub fn accumulator_width(&self) -> u8 {}

    pub fn index_width(&self) -> u8 {}

    pub fn is_native_mode(&self) -> bool {}

    pub fn is_emulation_mode(&self) -> bool {}

    // Address helpers

    pub fn program_address(&self) -> u32 {}

    pub fn data_address(&self, address: u16) -> u32 {}
}