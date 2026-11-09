use super::addressing_modes::{AddressingMode, EffectiveAddress};
use super::registers::Registers;
use crate::memory::bus::Bus;

impl AddressingMode {
    /// Reads the operand bytes after the opcode (advancing PC) and returns the address they point to.
    pub fn resolve(&self, regs: &mut Registers, bus: &mut Bus) -> EffectiveAddress {
        match self {
            AddressingMode::ImmediateA => {
                let address = regs.program_address(regs.pc());
                if regs.a_is_8bit() {
                    regs.increment_pc(1);
                } else {
                    regs.increment_pc(2);
                }
                EffectiveAddress {
                    address,
                    extra_cycles: 0,
                }
            }

            AddressingMode::ImmediateXY => {
                let address = regs.program_address(regs.pc());
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
                let offset = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                EffectiveAddress {
                    address: regs.data_address(offset),
                    extra_cycles: 0,
                }
            }

            AddressingMode::AbsoluteLong => {
                let offset = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                let bank = bus.read8(regs.program_address(regs.pc()));
                regs.increment_pc(1);
                EffectiveAddress {
                    address: (bank as u32) << 16 | (offset as u32),
                    extra_cycles: 0,
                }
            }

            AddressingMode::Direct => {
                let operand = fetch_operand8(regs, bus);
                EffectiveAddress {
                    address: regs.dp().wrapping_add(operand) as u32,
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndexedX => {
                let operand = fetch_operand8(regs, bus);
                let offset = regs.dp().wrapping_add(operand).wrapping_add(regs.index_x());
                EffectiveAddress {
                    address: offset as u32,
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndexedY => {
                let operand = fetch_operand8(regs, bus);
                let offset = regs.dp().wrapping_add(operand).wrapping_add(regs.index_y());
                EffectiveAddress {
                    address: offset as u32,
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndirect => {
                let operand = fetch_operand8(regs, bus);
                let ptr = regs.dp().wrapping_add(operand) as u32;
                let pointer = bus.read16_bank_wrap(ptr);
                EffectiveAddress {
                    address: regs.data_address(pointer),
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndirectLong => {
                let operand = fetch_operand8(regs, bus);
                let ptr = regs.dp().wrapping_add(operand) as u32;
                EffectiveAddress {
                    address: bus.read24_bank_wrap(ptr),
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndexedIndirect => {
                let operand = fetch_operand8(regs, bus);
                // Indexed first, then indirect: pointer at (D + d + X) in bank 0
                let ptr = regs.dp().wrapping_add(operand).wrapping_add(regs.index_x()) as u32;
                let pointer = bus.read16_bank_wrap(ptr);
                EffectiveAddress {
                    address: regs.data_address(pointer),
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndirectIndexed => {
                let operand = fetch_operand8(regs, bus);
                let ptr = regs.dp().wrapping_add(operand) as u32;
                let pointer = bus.read16_bank_wrap(ptr);
                // DBR:pointer + Y, carries into the next bank
                let effective = (regs.data_address(pointer) + regs.index_y() as u32) & 0xFF_FFFF;
                EffectiveAddress {
                    address: effective,
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::DirectIndirectLongIndexed => {
                let operand = fetch_operand8(regs, bus);
                let ptr = regs.dp().wrapping_add(operand) as u32;
                let base = bus.read24_bank_wrap(ptr);
                let effective = base.wrapping_add(regs.index_y() as u32) & 0x00FF_FFFF;
                EffectiveAddress {
                    address: effective,
                    extra_cycles: dl_penalty(regs),
                }
            }

            AddressingMode::AbsoluteIndexedX => {
                let base_address = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                let index = regs.index_x();
                // Page crossing costs a cycle, but only with 8-bit index registers
                let carry = base_address.overflowing_add(index).1;
                EffectiveAddress {
                    address: (regs.data_address(base_address) + index as u32) & 0xFF_FFFF,
                    extra_cycles: u8::from(regs.index_is_8bit() && carry),
                }
            }

            AddressingMode::AbsoluteIndexedY => {
                let base_address = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                let index = regs.index_y();
                let carry = base_address.overflowing_add(index).1;
                EffectiveAddress {
                    address: (regs.data_address(base_address) + index as u32) & 0xFF_FFFF,
                    extra_cycles: u8::from(regs.index_is_8bit() && carry),
                }
            }

            AddressingMode::AbsoluteLongIndexedX => {
                let base_address = bus.read24_bank_wrap(regs.program_address(regs.pc()));
                regs.increment_pc(3);
                let effective = base_address.wrapping_add(regs.index_x() as u32) & 0x00FF_FFFF;
                EffectiveAddress {
                    address: effective,
                    extra_cycles: 0,
                }
            }

            AddressingMode::AbsoluteIndexedIndirect => {
                let base_address = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                let ptr = base_address.wrapping_add(regs.index_x());
                let target_pc = bus.read16_bank_wrap(regs.program_address(ptr));
                EffectiveAddress {
                    address: regs.program_address(target_pc),
                    extra_cycles: 0,
                }
            }

            AddressingMode::AbsoluteIndirectLong => {
                let operand = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                EffectiveAddress {
                    address: bus.read24_bank_wrap(operand as u32),
                    extra_cycles: 0,
                }
            }

            AddressingMode::AbsoluteIndirect => {
                let operand = bus.read16(regs.program_address(regs.pc()));
                regs.increment_pc(2);
                let target = bus.read16_bank_wrap(operand as u32);
                EffectiveAddress {
                    address: regs.program_address(target),
                    extra_cycles: 0,
                }
            }

            AddressingMode::StackRelative => {
                let offset = fetch_operand8(regs, bus);
                EffectiveAddress {
                    address: regs.sp().wrapping_add(offset) as u32,
                    extra_cycles: 0,
                }
            }

            AddressingMode::StackRelativeIndexedIndirect => {
                let offset = fetch_operand8(regs, bus);
                let ptr = regs.sp().wrapping_add(offset) as u32;
                let pointer = bus.read16_bank_wrap(ptr);
                let effective = (regs.data_address(pointer) + regs.index_y() as u32) & 0xFF_FFFF;
                EffectiveAddress {
                    address: effective,
                    extra_cycles: 0,
                }
            }

            AddressingMode::Relative => {
                let offset = fetch_operand8(regs, bus) as i8;
                let target = regs.pc().wrapping_add(offset as u16);
                EffectiveAddress {
                    address: regs.program_address(target),
                    extra_cycles: 0,
                }
            }

            AddressingMode::RelativeLong => {
                let offset = bus.read16(regs.program_address(regs.pc())) as i16;
                regs.increment_pc(2);
                let target = regs.pc().wrapping_add(offset as u16);
                EffectiveAddress {
                    address: regs.program_address(target),
                    extra_cycles: 0,
                }
            }

            // No operand address: the instruction works on A, registers or the block move pointers
            AddressingMode::Accumulator | AddressingMode::Implied | AddressingMode::BlockMove => {
                EffectiveAddress {
                    address: 0,
                    extra_cycles: 0,
                }
            }
        }
    }
}

/// Reads the one-byte operand after the opcode and advances PC past it.
fn fetch_operand8(regs: &mut Registers, bus: &mut Bus) -> u16 {
    let operand = bus.read8(regs.program_address(regs.pc())) as u16;
    regs.increment_pc(1);
    operand
}

/// Direct-page modes cost one extra cycle when the low byte of D is not zero.
fn dl_penalty(regs: &Registers) -> u8 {
    u8::from(regs.dp() & 0x00FF != 0)
}
