use super::cpu::Cpu;
use crate::memory::bus::Bus;

// Every CPU read, write and stack access goes through the bus.
impl Cpu {
    /// Reads the byte at PB:PC and advances PC.
    pub(super) fn fetch8(&mut self, bus: &mut Bus) -> u8 {
        let v = bus.read8(self.regs.program_address(self.regs.pc()));
        self.regs.increment_pc(1);
        v
    }

    /// Reads one byte, or a little-endian word when `wide` is set.
    pub(super) fn read(&self, bus: &mut Bus, addr: u32, wide: bool) -> u16 {
        if wide {
            bus.read16(addr)
        } else {
            bus.read8(addr) as u16
        }
    }

    /// Writes one byte, or a little-endian word when `wide` is set.
    pub(super) fn write(&self, bus: &mut Bus, addr: u32, v: u16, wide: bool) {
        if wide {
            bus.write16(addr, v);
        } else {
            bus.write8(addr, v as u8);
        }
    }

    pub(super) fn push8(&mut self, bus: &mut Bus, v: u8) {
        bus.write8(self.regs.sp() as u32, v);
        self.set_sp(self.regs.sp().wrapping_sub(1));
    }

    pub(super) fn pull8(&mut self, bus: &mut Bus) -> u8 {
        self.set_sp(self.regs.sp().wrapping_add(1));
        bus.read8(self.regs.sp() as u32)
    }

    /// Pushes the high byte first, so the low byte ends up at the lower address.
    pub(super) fn push16(&mut self, bus: &mut Bus, v: u16) {
        self.push8(bus, (v >> 8) as u8);
        self.push8(bus, v as u8);
    }

    pub(super) fn pull16(&mut self, bus: &mut Bus) -> u16 {
        let low = self.pull8(bus) as u16;
        let high = self.pull8(bus) as u16;
        high << 8 | low
    }

    pub(super) fn push(&mut self, bus: &mut Bus, v: u16, wide: bool) {
        if wide {
            self.push16(bus, v);
        } else {
            self.push8(bus, v as u8);
        }
    }

    pub(super) fn pull(&mut self, bus: &mut Bus, wide: bool) -> u16 {
        if wide {
            self.pull16(bus)
        } else {
            self.pull8(bus) as u16
        }
    }
}
