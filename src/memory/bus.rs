pub struct Bus {
    wram: [u8; 0x20000], // 128KB of Work RAM
    rom: Vec<u8>,        // Cartridge ROM
                         // ppu: PPU, // Picture Processing Unit
                         // apu: APU, // Audio Processing Unit
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Bus {
            wram: [0; 0x20000], // Initialize Work RAM to zero
            rom,                // Load the provided ROM
                                // ppu: PPU::new(), // Initialize PPU
                                // apu: APU::new(), // Initialize APU
        }
    }
    // Read a byte (8 bits) from the bus at the specified address
    pub fn read8(&mut self, addr: u32) -> u8 {
        let bank = (addr >> 16) as u8; // Get the bank number (upper 8 bits)
        let offset = (addr & 0xFFFF) as u16; // Get the offset within the bank
        match (bank, offset) {
            // WRAM real, banks $7E-$7F
            (0x7E..=0x7F, _) => {
                let wram_addr = ((bank as u32 - 0x7E) << 16) | offset as u32; // Calculate the WRAM address
                self.wram[wram_addr as usize] // Return the byte from WRAM
            }

            // WRAM mirror, banks $00-$3F and $80-$BF. offset $0000-$1FFF
            (0x00..=0x3F | 0x80..=0xBF, 0x0000..=0x1FFF) => {
                self.wram[offset as usize] // Return the byte from WRAM mirror
            }

            // Rom (LoROM), banks $00-$7D and $80-$FF, offset $8000-$FFFF
            (0x00..=0x7D | 0x80..=0xFF, 0x8000..=0xFFFF) => {
                let rom_offset = ((bank as usize & 0x7F) * 0x8000) + (offset as usize - 0x8000); // Calculate the ROM offset
                self.rom[rom_offset] // Return the byte from ROM
            }

            _ => panic!("read8: unmapped address: {:#08X}", addr), // panic if address is unmapped
        }
    }
    pub fn write8(&mut self, addr: u32, value: u8) {
        let bank = (addr >> 16) as u8; // Get the bank number (upper 8 bits)
        let offset = (addr & 0xFFFF) as u16; // Get the offset within the bank

        match (bank, offset) {
            // WRAM real, banks $7E-$7F, offset $0000
            (0x7E..=0x7F, _) => {
                let wram_addr = ((bank as u32 - 0x7E) << 16) | offset as u32;
                self.wram[wram_addr as usize] = value; // Write to WRAM
            }
            // WRAM mirror, banks $00-$3F and $80-$BF. offset $0000-$1FFF
            (0x00..=0x3F | 0x80..=0xBF, 0x0000..=0x1FFF) => {
                self.wram[offset as usize] = value; // Write to WRAM
            }
            // Only read the rom, so no write part here
            _ => panic!("write8: unmapped address: {:#08X}", addr), // panic if address is unmapped
        }
    }

    pub fn read16(&mut self, addr: u32) -> u16 {
        let low = self.read8(addr) as u16; // Read the low byte
        let high = self.read8(addr.wrapping_add(1)) as u16; // Read the high byte

        (high << 8) | low // Combine into a 16-bit value
    }

    pub fn read16_bank_wrap(&mut self, addr: u32) -> u16 {
        let low = self.read8(addr) as u16; // Read the low byte
        let high = self.read8((addr & 0xFFFF_0000) | ((addr.wrapping_add(1)) & 0x0000_FFFF)) as u16; // Read the high byte with bank wrapping

        (high << 8) | low // Combine into a 16-bit value
    }

    pub fn read24_bank_wrap(&mut self, addr: u32) -> u32 {
        let low = self.read8(addr) as u32; // Read the low byte
        let high = self.read8((addr & 0xFFFF_0000) | ((addr.wrapping_add(1)) & 0x0000_FFFF)) as u32; // Read the high byte with bank wrapping
        let bank = self.read8((addr & 0xFFFF_0000) | ((addr.wrapping_add(2)) & 0x0000_FFFF)) as u32; // Read the bank byte with bank wrapping

        (bank << 16) | (high << 8) | low // Combine into a 24-bit value
    }

    pub fn write16(&mut self, addr: u32, value: u16) {
        let low = (value & 0xFF) as u8; // low byte
        let high = (value >> 8) as u8; // high byte

        self.write8(addr, low); // Write low byte
        self.write8(addr.wrapping_add(1), high); // Write high byte
    }
}


