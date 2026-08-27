//! Unit tests for the implemented 65816 addressing modes.
//!
//! These tests lock in the hardware behaviors corrected during development:
//! - Direct page always resides in bank $00 (regardless of DB)
//! - Extra cycle charged when D low byte is non-zero (DL != 0)
//! - Index registers masked to 8 bits when P.x = 1
//! - DirectIndirect reads its pointer from bank zero, final data from DB bank

use super::addressing_modes::AddressingMode;
use super::registers::Registers;
use crate::memory::bus::Bus;

/// Builds a Registers + Bus pair with a single operand byte placed at PC.
fn setup(operand: u8) -> (Registers, Bus) {
    let regs = Registers::new();
    let mut bus = Bus::new(vec![0; 0x8000]);
    // PB:PC resolves to bank $00 offset $0000 -> WRAM mirror region (mapped)
    bus.write8(regs.program_address(regs.pc()), operand);
    (regs, bus)
}

#[test]
fn direct_resolves_to_bank_zero_even_with_db_set() {
    let (mut regs, mut bus) = setup(0x10);
    regs.set_db(0x7E);

    let ea = AddressingMode::Direct.resolve(&mut regs, &mut bus);

    assert_eq!(ea.address, 0x0000_0010);
    assert_eq!(ea.extra_cycles, 0);
    assert_eq!(regs.pc(), 1);
}

#[test]
fn direct_charges_extra_cycle_when_dl_nonzero() {
    let (mut regs, mut bus) = setup(0x10);
    regs.set_dp(0x0001);

    let ea = AddressingMode::Direct.resolve(&mut regs, &mut bus);

    assert_eq!(ea.address, 0x0000_0011);
    assert_eq!(ea.extra_cycles, 1);
}

#[test]
fn direct_indexed_x_masks_high_byte_in_8bit_mode() {
    let (mut regs, mut bus) = setup(0x10);
    // P.x defaults to true (8-bit indexes); hidden high byte must be ignored
    regs.set_x(0x12AB);

    let ea = AddressingMode::DirectIndexedX.resolve(&mut regs, &mut bus);

    assert_eq!(ea.address, 0x0000_00BB);
    assert_eq!(ea.extra_cycles, 0);
}

#[test]
fn direct_indexed_x_uses_full_register_in_16bit_mode() {
    let (mut regs, mut bus) = setup(0x10);
    regs.status_mut().x = false;
    regs.set_x(0x12AB);

    let ea = AddressingMode::DirectIndexedX.resolve(&mut regs, &mut bus);

    assert_eq!(ea.address, 0x0000_12BB);
}

#[test]
fn direct_indexed_y_masks_high_byte_in_8bit_mode() {
    let (mut regs, mut bus) = setup(0x10);
    regs.set_y(0x12AB);

    let ea = AddressingMode::DirectIndexedY.resolve(&mut regs, &mut bus);

    assert_eq!(ea.address, 0x0000_00BB);
    assert_eq!(ea.extra_cycles, 0);
}

#[test]
fn direct_indexed_y_charges_extra_cycle_when_dl_nonzero() {
    let (mut regs, mut bus) = setup(0x10);
    regs.set_dp(0x0001);

    let ea = AddressingMode::DirectIndexedY.resolve(&mut regs, &mut bus);

    assert_eq!(ea.extra_cycles, 1);
}

#[test]
fn direct_indirect_reads_pointer_from_bank_zero_and_data_from_db_bank() {
    let (mut regs, mut bus) = setup(0x20);
    regs.set_db(0x7E);
    // Pointer word lives at $00:0020 (bank zero, WRAM mirror)
    bus.write16(0x0000_0020, 0xF800);

    let ea = AddressingMode::DirectIndirect.resolve(&mut regs, &mut bus);

    // Final data address folds DB, NOT the pointer location
    assert_eq!(ea.address, 0x007E_F800);
    assert_eq!(ea.extra_cycles, 0);
}
