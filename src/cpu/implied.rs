use super::cpu::Cpu;
use super::instructions::Instruction::{self, *};

impl Cpu {
    /// Runs an instruction that only touches registers, flags or the halt state.
    pub(super) fn run_implied(&mut self, instruction: Instruction) {
        match instruction {
            // Transfers
            TAX => self.set_x_nz(self.regs.a()),
            TAY => self.set_y_nz(self.regs.a()),
            TXA => self.set_a_nz(self.regs.x()),
            TYA => self.set_a_nz(self.regs.y()),
            TSX => self.set_x_nz(self.regs.sp()),
            TXS => self.set_sp(self.regs.x()),
            TXY => self.set_y_nz(self.regs.x()),
            TYX => self.set_x_nz(self.regs.y()),
            TCS => self.set_sp(self.regs.a()),
            TCD => {
                self.regs.set_dp(self.regs.a());
                self.set_nz(self.regs.a(), true);
            }
            TDC => {
                self.regs.set_a(self.regs.dp());
                self.set_nz(self.regs.a(), true);
            }
            TSC => {
                self.regs.set_a(self.regs.sp());
                self.set_nz(self.regs.a(), true);
            }
            XBA => {
                self.regs.set_a(self.regs.a().swap_bytes());
                self.set_nz(self.regs.a(), false);
            }

            // Increments and decrements
            INX => self.set_x_nz(self.regs.x().wrapping_add(1)),
            INY => self.set_y_nz(self.regs.y().wrapping_add(1)),
            DEX => self.set_x_nz(self.regs.x().wrapping_sub(1)),
            DEY => self.set_y_nz(self.regs.y().wrapping_sub(1)),

            // Flags
            CLC => self.regs.status_mut().c = false,
            SEC => self.regs.status_mut().c = true,
            CLD => self.regs.status_mut().d = false,
            SED => self.regs.status_mut().d = true,
            CLI => self.regs.status_mut().i = false,
            SEI => self.regs.status_mut().i = true,
            CLV => self.regs.status_mut().v = false,
            XCE => {
                let carry = self.regs.status().c;
                self.regs.status_mut().c = self.regs.emulation_mode();
                self.regs.set_emulation_mode(carry);
                self.set_p(self.regs.status().bits());
            }

            // Halts
            WAI => self.waiting = true,
            STP => self.stopped = true,

            _ => unreachable!("not an implied instruction: {instruction:?}"),
        }
    }
}
