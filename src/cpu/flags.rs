#[derive(Debug, Clone, Copy)]
pub struct StatusRegister {
    pub n: bool, // Negative Flag
    pub v: bool, // Overflow Flag
    pub m: bool, // Memory/Accumulator Flag
    pub x: bool, // Index Register Flag
    pub d: bool, // Decimal Mode Flag
    pub i: bool, // Interrupt Disable Flag
    pub z: bool, // Zero Flag
    pub c: bool, // Carry Flag
}