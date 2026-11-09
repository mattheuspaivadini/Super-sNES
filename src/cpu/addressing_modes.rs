/// The 65C816 addressing modes. `resolve.rs` turns each one into an address.
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
    AbsoluteLongIndexedX,
    StackRelative,
    StackRelativeIndexedIndirect,
    Accumulator,
    Implied,
    Relative,
    RelativeLong,
    BlockMove, // MVN/MVP
}

/// Where an operand lives, and the extra cycles its addressing mode costs.
pub struct EffectiveAddress {
    pub address: u32,
    pub extra_cycles: u8,
}
