//! Unit tests for CPU decode and execution.

use super::cpu::Cpu;
use super::opcodes::decode;
use crate::memory::bus::Bus;

const START: u16 = 0x1000; // program lives in the WRAM mirror of bank $00

/// Loads a program at $00:1000 and runs `steps` instructions.
fn run(program: &[u8], steps: usize) -> (Cpu, Bus) {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new(vec![0; 0x8000]);
    for (i, &byte) in program.iter().enumerate() {
        bus.write8(START as u32 + i as u32, byte);
    }
    cpu.regs.set_pc(START);
    for _ in 0..steps {
        cpu.step(&mut bus);
    }
    (cpu, bus)
}

#[test]
fn every_opcode_decodes() {
    for op in 0..=255u8 {
        decode(op);
    }
}

#[test]
fn lda_immediate_sets_flags() {
    let (cpu, _) = run(&[0xA9, 0x80], 1); // LDA #$80
    assert_eq!(cpu.regs.a(), 0x80);
    assert!(cpu.regs.status().n);
    assert!(!cpu.regs.status().z);
}

#[test]
fn native_mode_16bit_load_and_store() {
    // CLC; XCE; REP #$30; LDA #$1234; STA $20
    let (cpu, mut bus) = run(&[0x18, 0xFB, 0xC2, 0x30, 0xA9, 0x34, 0x12, 0x85, 0x20], 5);
    assert!(cpu.regs.is_native_mode());
    assert_eq!(cpu.regs.a(), 0x1234);
    assert_eq!(bus.read16(0x20), 0x1234);
}

#[test]
fn emulation_mode_forces_8bit_registers() {
    let (cpu, _) = run(&[0xC2, 0x30], 1); // REP #$30 while still in emulation
    assert!(cpu.regs.a_is_8bit());
    assert!(cpu.regs.index_is_8bit());
}

#[test]
fn adc_binary_sets_carry_and_overflow() {
    // CLC; LDA #$7F; ADC #$01 -> $80, V set
    let (cpu, _) = run(&[0x18, 0xA9, 0x7F, 0x69, 0x01], 3);
    assert_eq!(cpu.regs.a(), 0x80);
    assert!(cpu.regs.status().v);
    assert!(!cpu.regs.status().c);
}

#[test]
fn adc_decimal_mode() {
    // SED; CLC; LDA #$19; ADC #$03 -> $22
    let (cpu, _) = run(&[0xF8, 0x18, 0xA9, 0x19, 0x69, 0x03], 4);
    assert_eq!(cpu.regs.a(), 0x22);
}

#[test]
fn sbc_decimal_mode_borrows() {
    // SED; SEC; LDA #$10; SBC #$01 -> $09
    let (cpu, _) = run(&[0xF8, 0x38, 0xA9, 0x10, 0xE9, 0x01], 4);
    assert_eq!(cpu.regs.a(), 0x09);
    assert!(cpu.regs.status().c);
}

#[test]
fn lsr_in_8bit_mode_keeps_b_out() {
    let (mut cpu, mut bus) = run(&[0x4A], 0); // LSR A
    cpu.regs.set_a(0x0100); // B = $01, A low = $00
    cpu.step(&mut bus);
    assert_eq!(cpu.regs.a(), 0x0100);
    assert!(cpu.regs.status().z);
}

#[test]
fn branch_taken_and_not_taken() {
    // LDX #$02; DEX; BNE -3 -> loops until X = 0
    let (cpu, _) = run(&[0xA2, 0x02, 0xCA, 0xD0, 0xFD], 5);
    assert_eq!(cpu.regs.x(), 0);
    assert_eq!(cpu.regs.pc(), START + 5);
}

#[test]
fn jsr_and_rts_round_trip() {
    // JSR $1010; ... at $1010: RTS
    let mut program = vec![0x20, 0x10, 0x10];
    program.resize(0x10, 0xEA);
    program.push(0x60);
    let (cpu, _) = run(&program, 2);
    assert_eq!(cpu.regs.pc(), START + 3);
    assert_eq!(cpu.regs.sp(), 0x01FF);
}

#[test]
fn push_pull_status_round_trip() {
    // SEC; PHP; CLC; PLP
    let (cpu, _) = run(&[0x38, 0x08, 0x18, 0x28], 4);
    assert!(cpu.regs.status().c);
}

#[test]
fn xba_swaps_accumulator_bytes() {
    let (mut cpu, mut bus) = run(&[0xEB], 0);
    cpu.regs.set_a(0x12FF);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs.a(), 0xFF12);
    assert!(!cpu.regs.status().n);
}

#[test]
fn mvn_copies_block() {
    // CLC; XCE; REP #$30; LDA #$0002; LDX #$0100; LDY #$0200; MVN $00,$00
    let program = [
        0x18, 0xFB, 0xC2, 0x30, 0xA9, 0x02, 0x00, 0xA2, 0x00, 0x01, 0xA0, 0x00, 0x02, 0x54, 0x00,
        0x00,
    ];
    let (mut cpu, mut bus) = run(&program, 6);
    for i in 0..3 {
        bus.write8(0x0100 + i, 0xA0 + i as u8);
    }
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert_eq!([bus.read8(0x200), bus.read8(0x201), bus.read8(0x202)], [0xA0, 0xA1, 0xA2]);
    assert_eq!(cpu.regs.a(), 0xFFFF);
    assert_eq!(cpu.regs.pc(), START + program.len() as u16);
}

#[test]
fn brk_in_emulation_jumps_to_vector() {
    let mut rom = vec![0; 0x8000];
    rom[0x7FFE] = 0x00; // $00:FFFE -> handler at $9000
    rom[0x7FFF] = 0x90;
    let mut cpu = Cpu::new();
    let mut bus = Bus::new(rom);
    bus.write8(START as u32, 0x00); // BRK
    cpu.regs.set_pc(START);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs.pc(), 0x9000);
    assert!(cpu.regs.status().i);
    assert_eq!(bus.read16(0x01FE), START + 2); // return address skips signature
}

#[test]
fn reset_loads_vector() {
    let mut rom = vec![0; 0x8000];
    rom[0x7FFC] = 0x00;
    rom[0x7FFD] = 0x80;
    let mut cpu = Cpu::new();
    let mut bus = Bus::new(rom);
    cpu.reset(&mut bus);
    assert_eq!(cpu.regs.pc(), 0x8000);
    assert!(cpu.regs.is_emulation_mode());
}
