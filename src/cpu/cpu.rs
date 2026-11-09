use super::addressing_modes::AddressingMode::{self, *};
use super::status::StatusRegister;
use super::instructions::Instruction::{self, *};
use super::opcodes::decode;
use super::registers::Registers;
use crate::memory::bus::Bus;

// Interrupt vectors in bank $00: (native, emulation)
const COP_VECTOR: (u16, u16) = (0xFFE4, 0xFFF4);
const BRK_VECTOR: (u16, u16) = (0xFFE6, 0xFFFE);
const NMI_VECTOR: (u16, u16) = (0xFFEA, 0xFFFA);
const IRQ_VECTOR: (u16, u16) = (0xFFEE, 0xFFFE);
const RESET_VECTOR: u16 = 0xFFFC;

pub struct Cpu {
    pub regs: Registers,
    pub waiting: bool, // WAI: halted until an interrupt
    pub stopped: bool, // STP: halted until reset
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: Registers::new(),
            waiting: false,
            stopped: false,
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        self.regs.reset();
        self.regs.status_mut().i = true;
        self.regs.set_pc(bus.read16(RESET_VECTOR as u32));
        self.waiting = false;
        self.stopped = false;
    }

    pub fn nmi(&mut self, bus: &mut Bus) -> u8 {
        self.waiting = false;
        7 + self.interrupt(bus, NMI_VECTOR, false)
    }

    /// Returns 0 when masked by the I flag (a masked IRQ still wakes up WAI).
    pub fn irq(&mut self, bus: &mut Bus) -> u8 {
        self.waiting = false;
        if self.regs.status().i {
            return 0;
        }
        7 + self.interrupt(bus, IRQ_VECTOR, false)
    }

    /// Executes one instruction and returns the cycles it took.
    pub fn step(&mut self, bus: &mut Bus) -> u8 {
        if self.stopped || self.waiting {
            return 1;
        }
        let opcode = self.fetch8(bus);
        let (ins, mode, base_cycles) = decode(opcode);
        let ea = mode.resolve(&mut self.regs, bus);
        base_cycles + ea.extra_cycles + self.execute(bus, ins, mode, ea.address)
    }

    /// Runs the instruction and returns cycles beyond the base count.
    fn execute(&mut self, bus: &mut Bus, ins: Instruction, mode: AddressingMode, addr: u32) -> u8 {
        let m16 = !self.regs.a_is_8bit();
        let x16 = !self.regs.index_is_8bit();
        let m_cycle = u8::from(m16);
        let x_cycle = u8::from(x16);

        match ins {
            // Load and store
            LDA => {
                let v = self.read(bus, addr, m16);
                self.set_a_nz(v);
                m_cycle
            }
            LDX => {
                let v = self.read(bus, addr, x16);
                self.set_x_nz(v);
                x_cycle
            }
            LDY => {
                let v = self.read(bus, addr, x16);
                self.set_y_nz(v);
                x_cycle
            }
            STA => {
                self.write(bus, addr, self.regs.a(), m16);
                m_cycle
            }
            STX => {
                self.write(bus, addr, self.regs.x(), x16);
                x_cycle
            }
            STY => {
                self.write(bus, addr, self.regs.y(), x16);
                x_cycle
            }
            STZ => {
                self.write(bus, addr, 0, m16);
                m_cycle
            }

            // Transfers
            TAX => self.done(|c| c.set_x_nz(c.regs.a())),
            TAY => self.done(|c| c.set_y_nz(c.regs.a())),
            TXA => self.done(|c| c.set_a_nz(c.regs.x())),
            TYA => self.done(|c| c.set_a_nz(c.regs.y())),
            TSX => self.done(|c| c.set_x_nz(c.regs.sp())),
            TXY => self.done(|c| c.set_y_nz(c.regs.x())),
            TYX => self.done(|c| c.set_x_nz(c.regs.y())),
            TXS => self.done(|c| c.set_sp(c.regs.x())),
            TCS => self.done(|c| c.set_sp(c.regs.a())),
            TCD => self.done(|c| {
                c.regs.set_dp(c.regs.a());
                c.set_nz(c.regs.a(), true);
            }),
            TDC => self.done(|c| {
                c.regs.set_a(c.regs.dp());
                c.set_nz(c.regs.a(), true);
            }),
            TSC => self.done(|c| {
                c.regs.set_a(c.regs.sp());
                c.set_nz(c.regs.a(), true);
            }),
            XBA => self.done(|c| {
                c.regs.set_a(c.regs.a().swap_bytes());
                c.set_nz(c.regs.a(), false);
            }),

            // Arithmetic and logic
            ORA | AND | EOR => {
                let v = self.read(bus, addr, m16);
                let a = self.regs.a();
                self.set_a_nz(match ins {
                    ORA => a | v,
                    AND => a & v,
                    _ => a ^ v,
                });
                m_cycle
            }
            ADC => {
                let v = self.read(bus, addr, m16);
                self.adc(v);
                m_cycle
            }
            SBC => {
                let v = self.read(bus, addr, m16);
                self.sbc(v);
                m_cycle
            }
            CMP => {
                let v = self.read(bus, addr, m16);
                self.compare(self.regs.a(), v, m16);
                m_cycle
            }
            CPX => {
                let v = self.read(bus, addr, x16);
                self.compare(self.regs.x(), v, x16);
                x_cycle
            }
            CPY => {
                let v = self.read(bus, addr, x16);
                self.compare(self.regs.y(), v, x16);
                x_cycle
            }
            BIT => {
                let v = self.read(bus, addr, m16);
                let a = self.regs.a();
                let sign = sign_bit(m16);
                let p = self.regs.status_mut();
                p.z = v & a & mask(m16) == 0;
                if mode != ImmediateA {
                    p.n = v & sign != 0;
                    p.v = v & (sign >> 1) != 0;
                }
                m_cycle
            }
            INX => self.done(|c| c.set_x_nz(c.regs.x().wrapping_add(1))),
            INY => self.done(|c| c.set_y_nz(c.regs.y().wrapping_add(1))),
            DEX => self.done(|c| c.set_x_nz(c.regs.x().wrapping_sub(1))),
            DEY => self.done(|c| c.set_y_nz(c.regs.y().wrapping_sub(1))),

            // Read-modify-write: on A in Accumulator mode, else on memory
            ASL | LSR | ROL | ROR | INC | DEC | TRB | TSB => {
                if mode == Accumulator {
                    let r = self.modify(ins, self.regs.a(), m16);
                    self.set_a(r);
                    0
                } else {
                    let v = self.read(bus, addr, m16);
                    let r = self.modify(ins, v, m16);
                    self.write(bus, addr, r, m16);
                    2 * m_cycle
                }
            }

            // Branches
            BCC | BCS | BEQ | BNE | BMI | BPL | BVC | BVS | BRA | BRL => {
                let p = *self.regs.status();
                let taken = match ins {
                    BCC => !p.c,
                    BCS => p.c,
                    BEQ => p.z,
                    BNE => !p.z,
                    BMI => p.n,
                    BPL => !p.n,
                    BVC => !p.v,
                    BVS => p.v,
                    _ => true,
                };
                if taken {
                    self.regs.set_pc(addr as u16);
                }
                u8::from(taken)
            }

            // Jumps and subroutines
            JMP => self.done(|c| c.regs.set_pc(addr as u16)),
            JML => self.done(|c| c.jump_long(addr)),
            JSR => {
                self.push16(bus, self.regs.pc().wrapping_sub(1));
                self.regs.set_pc(addr as u16);
                0
            }
            JSL => {
                self.push8(bus, self.regs.pb());
                self.push16(bus, self.regs.pc().wrapping_sub(1));
                self.jump_long(addr);
                0
            }
            RTS => {
                let pc = self.pull16(bus);
                self.regs.set_pc(pc.wrapping_add(1));
                0
            }
            RTL => {
                let pc = self.pull16(bus);
                self.regs.set_pc(pc.wrapping_add(1));
                let pb = self.pull8(bus);
                self.regs.set_pb(pb);
                0
            }
            RTI => {
                let p = self.pull8(bus);
                self.set_p(p);
                let pc = self.pull16(bus);
                self.regs.set_pc(pc);
                if self.regs.is_native_mode() {
                    let pb = self.pull8(bus);
                    self.regs.set_pb(pb);
                    return 1;
                }
                0
            }

            // Stack
            PHA => {
                self.push(bus, self.regs.a(), m16);
                m_cycle
            }
            PHX => {
                self.push(bus, self.regs.x(), x16);
                x_cycle
            }
            PHY => {
                self.push(bus, self.regs.y(), x16);
                x_cycle
            }
            PLA => {
                let v = self.pull(bus, m16);
                self.set_a_nz(v);
                m_cycle
            }
            PLX => {
                let v = self.pull(bus, x16);
                self.set_x_nz(v);
                x_cycle
            }
            PLY => {
                let v = self.pull(bus, x16);
                self.set_y_nz(v);
                x_cycle
            }
            PHP => {
                self.push8(bus, self.regs.status().bits());
                0
            }
            PLP => {
                let p = self.pull8(bus);
                self.set_p(p);
                0
            }
            PHD => {
                self.push16(bus, self.regs.dp());
                0
            }
            PLD => {
                let v = self.pull16(bus);
                self.regs.set_dp(v);
                self.set_nz(v, true);
                0
            }
            PHB => {
                self.push8(bus, self.regs.db());
                0
            }
            PLB => {
                let v = self.pull8(bus);
                self.regs.set_db(v);
                self.set_nz(v as u16, false);
                0
            }
            PHK => {
                self.push8(bus, self.regs.pb());
                0
            }
            PEA | PER => {
                self.push16(bus, addr as u16);
                0
            }
            PEI => {
                let v = bus.read16_bank_wrap(addr);
                self.push16(bus, v);
                0
            }

            // Flags
            CLC => self.done(|c| c.regs.status_mut().c = false),
            SEC => self.done(|c| c.regs.status_mut().c = true),
            CLD => self.done(|c| c.regs.status_mut().d = false),
            SED => self.done(|c| c.regs.status_mut().d = true),
            CLI => self.done(|c| c.regs.status_mut().i = false),
            SEI => self.done(|c| c.regs.status_mut().i = true),
            CLV => self.done(|c| c.regs.status_mut().v = false),
            REP => {
                let bits = self.fetch8(bus);
                self.set_p(self.regs.status().bits() & !bits);
                0
            }
            SEP => {
                let bits = self.fetch8(bus);
                self.set_p(self.regs.status().bits() | bits);
                0
            }
            XCE => self.done(|c| {
                let carry = c.regs.status().c;
                c.regs.status_mut().c = c.regs.emulation_mode();
                c.regs.set_emulation_mode(carry);
                c.set_p(c.regs.status().bits());
            }),

            // Block move: copies one byte per step and repeats itself until A wraps to $FFFF
            MVN | MVP => {
                let dst_bank = self.fetch8(bus);
                let src_bank = self.fetch8(bus);
                let (x, y) = (self.regs.x(), self.regs.y());
                let byte = bus.read8((src_bank as u32) << 16 | x as u32);
                bus.write8((dst_bank as u32) << 16 | y as u32, byte);
                self.regs.set_db(dst_bank);
                let step = if ins == MVN { 1u16 } else { 0xFFFF }; // +1 or -1
                self.set_x(x.wrapping_add(step));
                self.set_y(y.wrapping_add(step));
                self.regs.set_a(self.regs.a().wrapping_sub(1));
                if self.regs.a() != 0xFFFF {
                    self.regs.decrement_pc(3);
                }
                0
            }

            // System
            BRK | COP => {
                self.fetch8(bus); // signature byte
                let vector = if ins == BRK { BRK_VECTOR } else { COP_VECTOR };
                self.interrupt(bus, vector, true)
            }
            WDM => {
                self.fetch8(bus);
                0
            }
            WAI => self.done(|c| c.waiting = true),
            STP => self.done(|c| c.stopped = true),
            NOP => 0,
        }
    }

    // --- Interrupts ---

    /// Pushes state and jumps to the vector. Returns the extra native-mode cycle.
    fn interrupt(&mut self, bus: &mut Bus, vector: (u16, u16), software: bool) -> u8 {
        let emulation = self.regs.is_emulation_mode();
        if !emulation {
            self.push8(bus, self.regs.pb());
        }
        self.push16(bus, self.regs.pc());
        let mut p = self.regs.status().bits();
        if emulation && !software {
            p &= !0x10; // B flag is clear for hardware interrupts
        }
        self.push8(bus, p);

        let status = self.regs.status_mut();
        status.i = true;
        status.d = false;
        self.regs.set_pb(0);
        let addr = if emulation { vector.1 } else { vector.0 };
        self.regs.set_pc(bus.read16(addr as u32));
        u8::from(!emulation)
    }

    // --- Arithmetic ---

    fn adc(&mut self, v: u16) {
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

    fn sbc(&mut self, v: u16) {
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

    fn compare(&mut self, reg: u16, v: u16, wide: bool) {
        let reg = reg & mask(wide);
        self.regs.status_mut().c = reg >= v;
        self.set_nz(reg.wrapping_sub(v), wide);
    }

    fn modify(&mut self, ins: Instruction, v: u16, wide: bool) -> u16 {
        let (mask, sign) = (mask(wide), sign_bit(wide));
        let v = v & mask;
        let a = self.regs.a() & mask;
        let carry = self.regs.status().c;
        let p = self.regs.status_mut();
        let r = match ins {
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

    // --- Register helpers (respect the M/X widths) ---

    fn set_nz(&mut self, v: u16, wide: bool) {
        let p = self.regs.status_mut();
        p.z = v & mask(wide) == 0;
        p.n = v & sign_bit(wide) != 0;
    }

    fn set_a(&mut self, v: u16) {
        if self.regs.a_is_8bit() {
            self.regs.set_a_low(v as u8);
        } else {
            self.regs.set_a(v);
        }
    }

    fn set_a_nz(&mut self, v: u16) {
        self.set_a(v);
        self.set_nz(v, !self.regs.a_is_8bit());
    }

    fn set_x(&mut self, v: u16) {
        self.regs.set_x(v & mask(!self.regs.index_is_8bit()));
    }

    fn set_y(&mut self, v: u16) {
        self.regs.set_y(v & mask(!self.regs.index_is_8bit()));
    }

    fn set_x_nz(&mut self, v: u16) {
        self.set_x(v);
        self.set_nz(v, !self.regs.index_is_8bit());
    }

    fn set_y_nz(&mut self, v: u16) {
        self.set_y(v);
        self.set_nz(v, !self.regs.index_is_8bit());
    }

    fn set_sp(&mut self, v: u16) {
        let v = if self.regs.is_emulation_mode() { 0x0100 | v & 0xFF } else { v };
        self.regs.set_sp(v);
    }

    /// Sets P and applies the hardware rules that follow from it.
    fn set_p(&mut self, bits: u8) {
        *self.regs.status_mut() = StatusRegister::from_bits(bits);
        if self.regs.is_emulation_mode() {
            self.regs.status_mut().m = true;
            self.regs.status_mut().x = true;
            self.set_sp(self.regs.sp());
        }
        if self.regs.index_is_8bit() {
            self.regs.set_x(self.regs.x() & 0xFF);
            self.regs.set_y(self.regs.y() & 0xFF);
        }
    }

    fn jump_long(&mut self, addr: u32) {
        self.regs.set_pb((addr >> 16) as u8);
        self.regs.set_pc(addr as u16);
    }

    /// Runs a register-only instruction that adds no cycles.
    fn done(&mut self, f: impl FnOnce(&mut Self)) -> u8 {
        f(self);
        0
    }

    // --- Bus helpers ---

    fn fetch8(&mut self, bus: &mut Bus) -> u8 {
        let v = bus.read8(self.regs.program_address(self.regs.pc()));
        self.regs.increment_pc(1);
        v
    }

    fn read(&self, bus: &mut Bus, addr: u32, wide: bool) -> u16 {
        if wide {
            bus.read16(addr)
        } else {
            bus.read8(addr) as u16
        }
    }

    fn write(&self, bus: &mut Bus, addr: u32, v: u16, wide: bool) {
        if wide {
            bus.write16(addr, v);
        } else {
            bus.write8(addr, v as u8);
        }
    }

    fn push8(&mut self, bus: &mut Bus, v: u8) {
        bus.write8(self.regs.sp() as u32, v);
        self.set_sp(self.regs.sp().wrapping_sub(1));
    }

    fn pull8(&mut self, bus: &mut Bus) -> u8 {
        self.set_sp(self.regs.sp().wrapping_add(1));
        bus.read8(self.regs.sp() as u32)
    }

    fn push16(&mut self, bus: &mut Bus, v: u16) {
        self.push8(bus, (v >> 8) as u8);
        self.push8(bus, v as u8);
    }

    fn pull16(&mut self, bus: &mut Bus) -> u16 {
        let low = self.pull8(bus) as u16;
        let high = self.pull8(bus) as u16;
        high << 8 | low
    }

    fn push(&mut self, bus: &mut Bus, v: u16, wide: bool) {
        if wide {
            self.push16(bus, v);
        } else {
            self.push8(bus, v as u8);
        }
    }

    fn pull(&mut self, bus: &mut Bus, wide: bool) -> u16 {
        if wide {
            self.pull16(bus)
        } else {
            self.pull8(bus) as u16
        }
    }
}

fn mask(wide: bool) -> u16 {
    if wide { 0xFFFF } else { 0x00FF }
}

fn mask32(wide: bool) -> u32 {
    mask(wide) as u32
}

fn sign_bit(wide: bool) -> u16 {
    if wide { 0x8000 } else { 0x0080 }
}

fn bit_count(wide: bool) -> u32 {
    if wide { 16 } else { 8 }
}
