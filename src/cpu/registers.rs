#[derive(Debug, Clone, InSaveState)]
pub struct Registers {
    pub a: u16, // Accumulator
    pub x: u16, // Index Register X
    pub y: u16, // Index Register Y
    pub pc: u16, // Program Counter
    pub dp: u16, // Direct Page Register
    pub db: u8, // Data Bank Register
    pub pb: u8, // Program Bank Register
    pub pc: u16, // Program Counter
    pub p: StatusRegister, // Processor Status Register
    pub e: bool, // Emulation Mode Flag
}