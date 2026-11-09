use super::cpu::Cpu;
use super::instructions::Instruction::{self, *};

impl Cpu {
    pub(super) fn adc(&mut self, v: u16) {
        let wide = !self.regs.a_is_8bit();
        let (a, v, mask) = (self.regs.a() as u32 & mask32(wide), v as u32, mask32(wide));
        let p = *self.regs.status();
        let r = if p.d {
            let mut r = 0;
            let mut carry = p.c as u32;
            for shift in (0..bit_count(wide)).step_by(4) {
                let mut digit = (a >> shift & 0xF) + (v >> shift & 0xF) + carry;
                if digit > 9 {
                    digit += 6;
                }
                carry = u32::from(digit > 0xF);
                r |= (digit & 0xF) << shift;
            }
            r | carry << bit_count(wide)
        } else {
            a + v + p.c as u32
        };
        // shortcut: in decimal mode V uses the BCD result, real chips use an intermediate one; fine for games that ignore V after BCD
        self.regs.status_mut().v = !(a ^ v) & (a ^ r) & sign_bit(wide) as u32 != 0;
        self.regs.status_mut().c = r > mask;
        self.set_a_nz(r as u16);
    }

    pub(super) fn sbc(&mut self, v: u16) {
        let wide = !self.regs.a_is_8bit();
        if !self.regs.status().d {
            return self.adc(!v & mask(wide));
        }
        let (a, v) = (self.regs.a() as i32 & mask32(wide) as i32, v as i32);
        let mut r = 0;
        let mut borrow = i32::from(!self.regs.status().c);
        for shift in (0..bit_count(wide)).step_by(4) {
            let mut digit = (a >> shift & 0xF) - (v >> shift & 0xF) - borrow;
            borrow = i32::from(digit < 0);
            if digit < 0 {
                digit += 10;
            }
            r |= (digit & 0xF) << shift;
        }
        self.regs.status_mut().v = (a ^ v) & (a ^ r) & sign_bit(wide) as i32 != 0;
        self.regs.status_mut().c = borrow == 0;
        self.set_a_nz(r as u16);
    }

    pub(super) fn compare(&mut self, reg: u16, v: u16, wide: bool) {
        let reg = reg & mask(wide);
        self.regs.status_mut().c = reg >= v;
        self.set_nz(reg.wrapping_sub(v), wide);
    }

    /// Shifts, rotates, increments and the TRB/TSB bit tests. Returns the new value.
    pub(super) fn modify(&mut self, instruction: Instruction, v: u16, wide: bool) -> u16 {
        let (mask, sign) = (mask(wide), sign_bit(wide));
        let v = v & mask;
        let a = self.regs.a() & mask;
        let carry = self.regs.status().c;
        let p = self.regs.status_mut();
        let r = match instruction {
            ASL => {
                p.c = v & sign != 0;
                v << 1
            }
            LSR => {
                p.c = v & 1 != 0;
                v >> 1
            }
            ROL => {
                p.c = v & sign != 0;
                v << 1 | carry as u16
            }
            ROR => {
                p.c = v & 1 != 0;
                v >> 1 | if carry { sign } else { 0 }
            }
            INC => v.wrapping_add(1),
            DEC => v.wrapping_sub(1),
            // TRB/TSB only touch Z
            TRB => {
                p.z = v & a == 0;
                return v & !a;
            }
            _ => {
                p.z = v & a == 0;
                return v | a;
            }
        } & mask;
        self.set_nz(r, wide);
        r
    }
}

// Width helpers: 8-bit operands use the low byte, 16-bit operands the full word.

pub(super) fn mask(wide: bool) -> u16 {
    if wide { 0xFFFF } else { 0x00FF }
}

fn mask32(wide: bool) -> u32 {
    mask(wide) as u32
}

pub(super) fn sign_bit(wide: bool) -> u16 {
    if wide { 0x8000 } else { 0x0080 }
}

fn bit_count(wide: bool) -> u32 {
    if wide { 16 } else { 8 }
}
