use super::alu::{mask, sign_bit};
use super::opcodes::decode;
use super::registers::Registers;
use super::status::StatusRegister;
use crate::memory::bus::Bus;

// Interrupt vectors in bank $00: (native, emulation)
pub(super) const COP_VECTOR: (u16, u16) = (0xFFE4, 0xFFF4);
pub(super) const BRK_VECTOR: (u16, u16) = (0xFFE6, 0xFFFE);
const NMI_VECTOR: (u16, u16) = (0xFFEA, 0xFFFA);
const IRQ_VECTOR: (u16, u16) = (0xFFEE, 0xFFFE);
const RESET_VECTOR: u16 = 0xFFFC;

/// The 65C816 CPU: its registers and the halt state set by WAI and STP.
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
        let (instruction, mode, base_cycles) = decode(opcode);
        let operand = mode.resolve(&mut self.regs, bus);
        base_cycles + operand.extra_cycles + self.execute(bus, instruction, mode, operand.address)
    }

    /// Pushes state and jumps to the vector. Returns the extra native-mode cycle.
    pub(super) fn interrupt(&mut self, bus: &mut Bus, vector: (u16, u16), software: bool) -> u8 {
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

    // Register writes. These respect the M and X width flags.

    pub(super) fn set_nz(&mut self, v: u16, wide: bool) {
        let p = self.regs.status_mut();
        p.z = v & mask(wide) == 0;
        p.n = v & sign_bit(wide) != 0;
    }

    pub(super) fn set_a(&mut self, v: u16) {
        if self.regs.a_is_8bit() {
            self.regs.set_a_low(v as u8);
        } else {
            self.regs.set_a(v);
        }
    }

    pub(super) fn set_a_nz(&mut self, v: u16) {
        self.set_a(v);
        self.set_nz(v, !self.regs.a_is_8bit());
    }

    pub(super) fn set_x(&mut self, v: u16) {
        self.regs.set_x(v & mask(!self.regs.index_is_8bit()));
    }

    pub(super) fn set_y(&mut self, v: u16) {
        self.regs.set_y(v & mask(!self.regs.index_is_8bit()));
    }

    pub(super) fn set_x_nz(&mut self, v: u16) {
        self.set_x(v);
        self.set_nz(v, !self.regs.index_is_8bit());
    }

    pub(super) fn set_y_nz(&mut self, v: u16) {
        self.set_y(v);
        self.set_nz(v, !self.regs.index_is_8bit());
    }

    /// In emulation mode the stack pointer stays in page 1.
    pub(super) fn set_sp(&mut self, v: u16) {
        let v = if self.regs.is_emulation_mode() { 0x0100 | v & 0xFF } else { v };
        self.regs.set_sp(v);
    }

    /// Sets P and applies the hardware rules that follow from it.
    pub(super) fn set_p(&mut self, bits: u8) {
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

    pub(super) fn jump_long(&mut self, addr: u32) {
        self.regs.set_pb((addr >> 16) as u8);
        self.regs.set_pc(addr as u16);
    }
}
