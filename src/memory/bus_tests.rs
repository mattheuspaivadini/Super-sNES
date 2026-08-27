//! Unit tests for Bus word-read helpers.

use super::bus::Bus;

#[test]
fn read16_bank_wrap_reads_consecutive_bytes() {
    // LoROM mapping: bank $00 offset $8000+ -> rom[offset - 0x8000]
    let mut bus = Bus::new(vec![0x34, 0x12]);

    assert_eq!(bus.read16_bank_wrap(0x0000_8000), 0x1234);
}

#[test]
fn read16_bank_wrap_wraps_at_end_of_bank_zero() {
    // Low byte from last mapped ROM byte of bank $00 ($00:FFFF)
    let mut rom = vec![0x00; 0x8000];
    rom[0x7FFF] = 0x78;
    let mut bus = Bus::new(rom);
    // Wrapped high byte must come from $00:0000 (same bank, WRAM mirror),
    // NOT from $01:0000 (next bank).
    bus.write8(0x0000_0000, 0x9A);

    assert_eq!(bus.read16_bank_wrap(0x0000_FFFF), 0x9A78);
}

#[test]
fn read16_bank_wrap_preserves_bank_and_wraps_offset() {
    let mut bus = Bus::new(vec![0; 0x8000]);
    // Low byte from $7F:FFFF (WRAM), wrapped high byte from $7F:0000 (same bank)
    bus.write8(0x007F_FFFF, 0x11);
    bus.write8(0x007F_0000, 0x22);

    // A buggy implementation that crosses into the next bank would read
    // $80:0000 (WRAM mirror, value 0x00) instead of $7F:0000.
    assert_eq!(bus.read16_bank_wrap(0x007F_FFFF), 0x2211);
}
