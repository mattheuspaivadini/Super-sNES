use super::registers::Registers;
use crate::memory::bus::Bus;

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

            AddressingMode::DirectIndirectLongIndexed => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16;
                regs.increment_pc(1);
                let ptr = regs.dp().wrapping_add(operand) as u32; // Calculate the pointer address
                let base = bus.read24_bank_wrap(ptr); // Read the 24-bit pointer from memory (includes bank)
                let index = if regs.index_is_8bit() {
                    regs.y_low() as u32
                } else {
                    regs.y() as u32
                };
                let effective = base.wrapping_add(index) & 0x00FF_FFFF; // 24-bit address wrap
                EffectiveAddress {
                    address: effective,
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Extra cycle if DL != 0
                }
            }

            AddressingMode::DirectIndirectIndexed => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16;
                regs.increment_pc(1);
                let ptr = regs.dp().wrapping_add(operand) as u32; // Calculate the pointer address
                let base = bus.read24_bank_wrap(ptr); // Read the 24-bit pointer from memory (includes bank)
                let index = if regs.index_is_8bit() {
                    regs.x_low() as u32
                } else {
                    regs.x() as u32
                };
                let effective = base.wrapping_add(index) & 0x00FF_FFFF; // 24-bit address wrap
                EffectiveAddress {
                    address: effective,
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Extra cycle if DL != 0
                }
            }

            AddressingMode::DirectIndexedIndirect => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16;
                regs.increment_pc(1);
                let ptr = regs.dp().wrapping_add(operand) as u32; // Calculate the pointer address
                let base = bus.read24_bank_wrap(ptr); // Read the 24-bit pointer from memory (includes bank)
                let index = if regs.index_is_8bit() {
                    regs.x_low() as u32
                } else {
                    regs.x() as u32
                };
                let effective = base.wrapping_add(index) & 0x00FF_FFFF; // 24-bit address wrap
                EffectiveAddress {
                    address: effective,
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0), // Extra cycle if DL != 0
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

            AddressingMode::DirectIndexedIndirect => {
                let dp = bus.read8(regs.program_address(regs.pc())) as u16; // Read the direct page offset
                regs.increment_pc(1);

                let index = if (regs.dp() & 0x00FF != 0) { 1 } else { 0 }; // Determine the index (1 if direct page is not zero, 0 otherwise)
                let x_val = if regs.index_is_8bit() {
                    regs.x_low() as u16
                } else {
                    regs.x()
                }; // Get the X value (low byte if 8-bit index)
                   // Pointer always points to the low byte of the pointer value (D + dp + X) & 0xFFFF
                let ptr = regs
                    .dp()
                    .wrapping_add(dp)
                    .wrapping_add(x_val)
                    .wrapping_add(index) as u32; // Calculate the pointer address
                let pointer = bus.read16_bank_wrap(ptr); // Read the 16-bit pointer from memory

                EffectiveAddress {
                    address: regs.data_address(pointer),
                    extra_cycles: u8::from(regs.dp() & 0x00FF != 0),
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

            AddressingMode::AbsoluteIndirectLong => {
                // Read the 16-bit pointer address from the instruction stream
                let operand = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);

                // Dereference: read a 24-bit target address from bank 0 (with bank wrapping)
                let target = bus.read24_bank_wrap(operand as u32);

                EffectiveAddress {
                    address: target, // Full 24-bit address (bank included from memory)
                    extra_cycles: 0,
                }
            }

            AddressingMode::AbsoluteIndirect => {
                let operand = bus.read16(regs.program_address(regs.pc())); // Read the 16-bit pointer address
                regs.increment_pc(2); // Advance the PC past the operand
                                      // Dereference the pointer in bank 0 (with bank wrapping)
                let target = bus.read16_bank_wrap(operand as u32);
                EffectiveAddress {
                    address: regs.program_address(target), // PB:target
                    extra_cycles: 0,
                }
            }

            AddressingMode::StackRelative => {
                let offset = bus.read(regs.program_address(regs.pc())); // Read the offset from memory
                regs.advance_pc(1); // Advance the PC to the next instruction

                let stack_addr = regs.sp().wrapping_add(offset as u16); // Calculate the stack address
                let effective_addr = (0x00 << 16) | bus.read16(stack_addr); // Read the effective address from the stack

                let value = bus.read16(effective_addr); // Read the value from the effective address
                EffectiveAddress {
                    address: value,
                    extra_cycles: 0,
                }
            }

            AddressingMode::StackRelativeIndexedIndirect => {
                let offset = bus.read8(regs.program_address(regs.pc())); // Read 8-bit offset from instruction stream
                regs.increment_pc(1); // Advance the PC past the operand

                let ptr = regs.sp().wrapping_add(offset as u16) as u32; // SP + offset (bank 0)
                let pointer = bus.read16_bank_wrap(ptr); // Dereference: read 16-bit base address from [SP+d]

                // Add Y index
                let index = if regs.index_is_8bit() {
                    regs.y_low() as u16
                } else {
                    regs.y()
                };

                let effective = pointer.wrapping_add(index);
                EffectiveAddress {
                    address: regs.data_address(effective), // Resolve in DBR bank
                    extra_cycles: 0,
                }
            }

            AddressingMode::Accumulator => {
                EffectiveAddress {
                    address: 0, // No memory access
                    extra_cycles: 0,
                }
            }

            AddressingMode::Implied => {
                EffectiveAddress {
                    address: 0, // No memory access
                    extra_cycles: 0,
                }
            }

            AddressingMode::Relative => {
                let offset = bus.read8(regs.program_address(regs.pc())) as i8; // Read the signed 8-bit offset
                regs.increment_pc(1); // Advance the PC past the operand
                let target = regs.pc().wrapping_add(offset as u16); // PC (after fetch) + signed offset
                EffectiveAddress {
                    address: regs.program_address(target), // PB:target
                    extra_cycles: 0,
                }
            }

            AddressingMode::RelativeLong => {
                let offset = bus.read16(regs.program_address(regs.pc()) as u32) as i16; // Read the signed 16-bit offset
                regs.increment_pc(2); // Advance the PC past the 2-byte operand
                let target = regs.pc().wrapping_add(offset as u16); // PC (after fetch) + signed offset
                EffectiveAddress {
                    address: regs.program_address(target), // PB:target
                    extra_cycles: 0,
                }
            }

            AddressingMode::BlockMove => {
                EffectiveAddress {
                    address: 0, // No memory access
                    extra_cycles: 0,
                }
            }

            _ => {
                panic!("Addressing mode {:?} not implemented yet", self);
            }
        }
    }
}
