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

    // Pack as NVMXDIZC (bit 7 -> bit 0)
    pub fn bits(&self) -> u8 {
        [self.c, self.z, self.i, self.d, self.x, self.m, self.v, self.n]
            .iter()
            .enumerate()
            .fold(0, |acc, (bit, &set)| acc | (u8::from(set) << bit))
    }

    pub fn from_bits(value: u8) -> Self {
        let bit = |n: u8| value & (1 << n) != 0;
        StatusRegister {
            c: bit(0),
            z: bit(1),
            i: bit(2),
            d: bit(3),
            x: bit(4),
            m: bit(5),
            v: bit(6),
            n: bit(7),
        }
    }
}