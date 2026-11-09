use super::addressing_modes::AddressingMode::{self, *};
use super::alu::{mask, sign_bit};
use super::cpu::{BRK_VECTOR, COP_VECTOR, Cpu};
use super::instructions::Instruction::{self, *};
use crate::memory::bus::Bus;

impl Cpu {
    /// Runs one decoded instruction and returns the cycles beyond its base count.
    pub(super) fn execute(
        &mut self,
        bus: &mut Bus,
        instruction: Instruction,
        mode: AddressingMode,
        addr: u32,
    ) -> u8 {
        let a_wide = !self.regs.a_is_8bit();
        let x_wide = !self.regs.index_is_8bit();
        let a_cycle = u8::from(a_wide);
        let x_cycle = u8::from(x_wide);

        match instruction {
            // Load and store
            LDA => {
                let v = self.read(bus, addr, a_wide);
                self.set_a_nz(v);
                a_cycle
            }
            LDX => {
                let v = self.read(bus, addr, x_wide);
                self.set_x_nz(v);
                x_cycle
            }
            LDY => {
                let v = self.read(bus, addr, x_wide);
                self.set_y_nz(v);
                x_cycle
            }
            STA => {
                self.write(bus, addr, self.regs.a(), a_wide);
                a_cycle
            }
            STX => {
                self.write(bus, addr, self.regs.x(), x_wide);
                x_cycle
            }
            STY => {
                self.write(bus, addr, self.regs.y(), x_wide);
                x_cycle
            }
            STZ => {
                self.write(bus, addr, 0, a_wide);
                a_cycle
            }

            // Register-only instructions: transfers, increments, flags, WAI, STP
            TAX | TAY | TXA | TYA | TSX | TXS | TXY | TYX | TCS | TCD | TDC | TSC | XBA
            | INX | INY | DEX | DEY | CLC | SEC | CLD | SED | CLI | SEI | CLV | XCE | WAI
            | STP => {
                self.run_implied(instruction);
                0
            }

            // Arithmetic and logic
            ORA | AND | EOR => {
                let v = self.read(bus, addr, a_wide);
                let a = self.regs.a();
                self.set_a_nz(match instruction {
                    ORA => a | v,
                    AND => a & v,
                    _ => a ^ v,
                });
                a_cycle
            }
            ADC => {
                let v = self.read(bus, addr, a_wide);
                self.adc(v);
                a_cycle
            }
            SBC => {
                let v = self.read(bus, addr, a_wide);
                self.sbc(v);
                a_cycle
            }
            CMP => {
                let v = self.read(bus, addr, a_wide);
                self.compare(self.regs.a(), v, a_wide);
                a_cycle
            }
            CPX => {
                let v = self.read(bus, addr, x_wide);
                self.compare(self.regs.x(), v, x_wide);
                x_cycle
            }
            CPY => {
                let v = self.read(bus, addr, x_wide);
                self.compare(self.regs.y(), v, x_wide);
                x_cycle
            }
            BIT => {
                let v = self.read(bus, addr, a_wide);
                let a = self.regs.a();
                let sign = sign_bit(a_wide);
                let p = self.regs.status_mut();
                p.z = v & a & mask(a_wide) == 0;
                if mode != ImmediateA {
                    p.n = v & sign != 0;
                    p.v = v & (sign >> 1) != 0;
                }
                a_cycle
            }

            // Read-modify-write: on A in Accumulator mode, else on memory
            ASL | LSR | ROL | ROR | INC | DEC | TRB | TSB => {
                if mode == Accumulator {
                    let r = self.modify(instruction, self.regs.a(), a_wide);
                    self.set_a(r);
                    0
                } else {
                    let v = self.read(bus, addr, a_wide);
                    let r = self.modify(instruction, v, a_wide);
                    self.write(bus, addr, r, a_wide);
                    2 * a_cycle
                }
            }

            // Branches
            BCC | BCS | BEQ | BNE | BMI | BPL | BVC | BVS | BRA | BRL => {
                let p = *self.regs.status();
                let taken = match instruction {
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
            JMP => {
                self.regs.set_pc(addr as u16);
                0
            }
            JML => {
                self.jump_long(addr);
                0
            }
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
            PHA | PLA | PHX | PLX | PHY | PLY | PHP | PLP | PHD | PLD | PHB | PLB | PHK | PEA
            | PER | PEI => self.run_stack(bus, instruction, addr),

            // Processor status bits
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

            // Block move: copies one byte per step and repeats itself until A wraps to $FFFF
            MVN | MVP => {
                let dst_bank = self.fetch8(bus);
                let src_bank = self.fetch8(bus);
                let (x, y) = (self.regs.x(), self.regs.y());
                let byte = bus.read8((src_bank as u32) << 16 | x as u32);
                bus.write8((dst_bank as u32) << 16 | y as u32, byte);
                self.regs.set_db(dst_bank);
                let step = if instruction == MVN { 1u16 } else { 0xFFFF }; // +1 or -1
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
                let vector = if instruction == BRK { BRK_VECTOR } else { COP_VECTOR };
                self.interrupt(bus, vector, true)
            }
            WDM => {
                self.fetch8(bus);
                0
            }
            NOP => 0,
        }
    }
}
