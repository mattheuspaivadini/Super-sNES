use super::cpu::Cpu;
use super::instructions::Instruction::{self, *};
use crate::memory::bus::Bus;

impl Cpu {
    /// Runs a push, pull or effective-address push. Returns the extra cycles.
    pub(super) fn run_stack(&mut self, bus: &mut Bus, instruction: Instruction, addr: u32) -> u8 {
        let a_wide = !self.regs.a_is_8bit();
        let x_wide = !self.regs.index_is_8bit();
        let a_cycle = u8::from(a_wide);
        let x_cycle = u8::from(x_wide);

        match instruction {
            PHA => {
                self.push(bus, self.regs.a(), a_wide);
                a_cycle
            }
            PLA => {
                let v = self.pull(bus, a_wide);
                self.set_a_nz(v);
                a_cycle
            }
            PHX => {
                self.push(bus, self.regs.x(), x_wide);
                x_cycle
            }
            PLX => {
                let v = self.pull(bus, x_wide);
                self.set_x_nz(v);
                x_cycle
            }
            PHY => {
                self.push(bus, self.regs.y(), x_wide);
                x_cycle
            }
            PLY => {
                let v = self.pull(bus, x_wide);
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
            _ => unreachable!("not a stack instruction: {instruction:?}"),
        }
    }
}
