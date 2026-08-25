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
    DirectIndirectLonge,
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

impl AddressingMode {
    pub fn resolve(&self, regs: &mut Registers, bus: &mut Bus) -> EffectiveAddress {
        match self {
            AddressingMode::ImmediateA => {
                let address = regs.program_address(regs.pc()) as u32;
                if regs.a_is_8bit() {
                regs.increment_pc(1); // Immediate mode uses the next byte as the operand
                } else { regs.increment_pc(2); }
                EffectiveAddress { address, extra_cycles: 0 }
            }

            AddressingMode::ImmediateXY => {
                let address = regs.program_address(regs.pc()) as u32;
                if regs.index_is_8bit() {
                    regs.increment_pc(1);
                } else { regs.increment_pc(2); }
                EffectiveAddress { address, extra_cycles: 0 }
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
                    address: regs.data_address(offset),
                    extra_cycles: 0,
                }
            }

            AddressingMode::DirectIndexedX => {
                let operand = bus.read8(regs.program_address(regs.pc())) as u16; // Read the direct page offset
                regs.increment_pc(1);
                let index = if regs.index_is_8bit() { regs.x_low() as u16 } else { regs.x() }; // Get the X register value, considering its size
                let offset = regs.dp().wrapping_add(operand).wrapping_add(index); // Calculate the effective address
                EffectiveAddress {
                    address: regs.data_address(offset),
                    extra_cycles: u8::from(regs.dp() & 0xFF00 != 0), // Add an extra cycle if the direct page is not zero
                }
            }
            _ => unimplemented!("Addressing mode {:?} not implemented yet", self),
        }
    }
    }