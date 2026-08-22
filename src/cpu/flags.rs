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

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister {
            n: false,
            v: false,
            m: true, // Default to 8-bit mode
            x: true, // X/Y default to 8-bit mode
            d: false,
            i: false,
            z: false,
            c: false,
        }
    }

    pub fn reset(&mut self) {
        self.n = false;
        self.v = false;
        self.m = true; // Reset to 8-bit mode
        self.x = true; // X/Y reset to 8-bit mode
        self.d = false;
        self.i = false;
        self.z = false;
        self.c = false;
    }
}