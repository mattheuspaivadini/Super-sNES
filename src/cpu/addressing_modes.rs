pub enum AddressingMode {
    Immediate,
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

pub fn resolve(
    mode: AddressingMode,
    regs: &Registers,
    bus: &mut Bus,
) -> EffectiveAddress

