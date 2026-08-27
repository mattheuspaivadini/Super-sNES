use super::registers::Registers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    ImmediateA,
    ImmediateXY,
    Absolute,
    AbsoluteLong,
    Direct,
    DirectIndexedX,
    DirectIndexedY,
    DirectIndirect,
    DirectIndirectLong,
    DirectIndexedIndirect,
    DirectIndirectIndexed,
    DirectIndirectLongIndexed,
    AbsoluteIndexedX,
    AbsoluteIndexedY,
    AbsoluteIndirect,
    AbsoluteIndirectLong,
    AbsoluteIndexedIndirect,
    StackRelative,
    StackRelativeIndexedIndirect,
    Accumulator,
    Implied,
    Relative,
    RelativeLong,
    BlockMove, // MVN/MVP
}

pub struct EffectiveAddress {
    pub address: u32,
    pub extra_cycles: u8,
}

// Implement the AddressingMode enum
// Implement the resolve method for AddressingMode
impl AddressingMode {
    pub fn resolve(&self, regs: &mut Registers, bus: &mut Bus) -> EffectiveAddress {
        match self {
            AddressingMode::ImmediateA => {
                let address = regs.program_address(regs.pc()) as u32;
                if regs.a_is_8bit() {
                    regs.increment_pc(1); // Immediate mode uses the next byte as the operand
                } else {
                    regs.increment_pc(2);
                }
                EffectiveAddress {
                    address,
                    extra_cycles: 0,
                }
            }

            AddressingMode::ImmediateXY => {
                let address = regs.program_address(regs.pc()) as u32;
                if regs.index_is_8bit() {
                    regs.increment_pc(1);
                } else {
                    regs.increment_pc(2);
                }
                EffectiveAddress {
                    address,
                    extra_cycles: 0,
                }
            }

            AddressingMode::Absolute => {
                let offset = bus.read16(regs.program_address(regs.pc()) as u32);
                regs.increment_pc(2);
                let address = regs.data_address(offset);
                EffectiveAddress {
                    address,
                    extra_cycles: 0,
                }
            }

            AddressingMode::AbsoluteLong => {
                let offset = bus.read16(regs.program_address(regs.pc()) as u32);
                regs.increment_pc(2);
                let bank = bus.read8(regs.program_address(regs.pc()));
                regs.increment_pc(1);
                let address = (bank as u32) << 16 | (offset as u32);
                EffectiveAddress {
                    address,
                    extra_cycles: 0,
                }
            }

            AddressingMode::Direct => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16;
                regs.increment_pc(1);
                let offset = regs.dp().wrapping_add(operand);
                EffectiveAddress {
                    address: offset as u32,
                    extra_cycles: u8::from(regs.dp() & 0xFF00 != 0), // Add an extra cycle if the direct page is not zero
                }
            }

            AddressingMode::DirectIndexedX => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16; // Read the direct page offset
                regs.increment_pc(1);
                let index = if regs.index_is_8bit() {
                    regs.x_low() as u16
                } else {
                    regs.x()
                }; // Get the X register value, considering its size
                let offset = regs.dp().wrapping_add(operand).wrapping_add(index); // Calculate the effective address
                EffectiveAddress {
                    address: offset as u32,                          // address to zero
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Add an extra cycle if the direct page is not zero
                }
            }

            AddressingMode::DirectIndexedY => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16; // Read the direct page offset
                regs.increment_pc(1);
                let index = if regs.index_is_8bit() {
                    regs.y_low() as u16
                } else {
                    regs.y()
                }; // Get the Y register value, considering its size
                let offset = regs.dp().wrapping_add(operand).wrapping_add(index); // Calculate the effective address
                EffectiveAddress {
                    address: offset as u32,
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Add an extra cycle if the direct page is not zero
                }
            }

            AddressingMode::DirectIndirect => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16; // Read the direct page offset
                regs.increment_pc(1);
                let ptr = regs.dp().wrapping_add(operand) as u32; // Calculate the pointer address
                let pointer = bus.read16_bank_wrap(ptr); // Read the 16-bit pointer from memory
                EffectiveAddress {
                    address: regs.data_address(pointer), // Resolve the pointer to get the effective address
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Add an extra cycle if the direct page is not zero
                }
            }

            AddressingMode::DirectIndirectLong => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16; // Read the direct page offset
                regs.increment_pc(1);
                let ptr = regs.dp().wrapping_add(operand) as u32; // Calculate the pointer address
                let pointer = bus.read24_bank_wrap(ptr); // Read the 24-bit pointer from memory
                EffectiveAddress {
                    address: pointer, // The effective address is the 24-bit pointer
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Add an extra cycle if the direct page is not zero
                }
            }

            AddressingMode::AbsoluteIndexedIndirect => {
                let base_address = bus.read16(regs.program_address(regs.pc())); // Read the base address from memory
                regs.increment_pc(2);

                let index = if regs.index_is_8bit() {
                    regs.x_low() as u16
                } else {
                    regs.x()
                };

                let ptr = base_address.wrapping_add(index); // Calculate the pointer address
                let ptr_address = regs.program_address(ptr); // Calculate the pointer address in the program bank
                let target_pc = bus.read16_bank_wrap(ptr_address); // Read the target PC from memory

                EffectiveAddress {
                    address: regs.program_address(target_pc),
                    extra_cycles: 0,
                }
            }

            AddressingMode::StackRelative => {
                let offset = bus.read(regs.program_address(regs.pc())); // Read the offset from memory
                regs.advance_pc(1);                                     // Advance the PC to the next instruction
                
                let stack_addr = regs.sp().wrapping_add(offset as u16); // Calculate the stack address
                let effective_addr = (0x00 << 16) | bus.read16(stack_addr); // Read the effective address from the stack

                let value = bus.read16(effective_addr);
                EffectiveAddress {
                    address: value,
                    extra_cycles: 0,
                }
            }
            _ => {
                panic!("Addressing mode {:?} not implemented yet", self);
            }
        }
    }
}
