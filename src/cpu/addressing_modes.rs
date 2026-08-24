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
    pub fn resolve(&self, regs: &Registers, bus: &mut Bus) -> EffectiveAddress {
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
            _ => unimplemented!("Addressing mode {:?} not implemented yet", self),
        }
    }
    }}
