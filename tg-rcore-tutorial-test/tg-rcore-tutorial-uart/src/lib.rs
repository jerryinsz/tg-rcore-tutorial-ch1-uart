#![no_std]
#![deny(warnings)]

use core::{hint::spin_loop, ptr};

pub const UART0_BASE: usize = 0x1000_0000;

const RBR: usize = 0;
const THR: usize = 0;
const IER: usize = 1;
const LSR: usize = 5;
const LSR_DR: u8 = 1 << 0;
const LSR_THRE: u8 = 1 << 5;

pub struct Uart16550 {
    base: usize,
}

impl Uart16550 {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    pub const fn qemu_virt() -> Self {
        Self::new(UART0_BASE)
    }

    pub fn init(&self) {
        unsafe { ptr::write_volatile((self.base + IER) as *mut u8, 0) };
    }

    pub fn putchar(&self, c: u8) {
        putchar_at(self.base, c);
    }

    pub fn putstr(&self, s: &str) {
        for b in s.bytes() {
            self.putchar(b);
        }
    }

    pub fn getchar(&self) -> Option<u8> {
        getchar_at(self.base)
    }
}

fn putchar_at(base: usize, c: u8) {
    while unsafe { ptr::read_volatile((base + LSR) as *const u8) } & LSR_THRE == 0 {
        spin_loop();
    }
    unsafe { ptr::write_volatile((base + THR) as *mut u8, c) };
}

fn getchar_at(base: usize) -> Option<u8> {
    if unsafe { ptr::read_volatile((base + LSR) as *const u8) } & LSR_DR == 0 {
        None
    } else {
        Some(unsafe { ptr::read_volatile((base + RBR) as *const u8) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn putchar_writes_to_thr() {
        let mut regs = [0u8; 8];
        regs[LSR] = LSR_THRE;
        putchar_at(regs.as_mut_ptr() as usize, b'A');
        assert_eq!(regs[THR], b'A');
    }

    #[test]
    fn putstr_writes_last_byte() {
        let mut regs = [0u8; 8];
        regs[LSR] = LSR_THRE;
        let uart = Uart16550::new(regs.as_mut_ptr() as usize);
        uart.putstr("Hi");
        assert_eq!(regs[THR], b'i');
    }

    #[test]
    fn init_disables_interrupt() {
        let mut regs = [0u8; 8];
        regs[IER] = 0xff;
        let uart = Uart16550::new(regs.as_mut_ptr() as usize);
        uart.init();
        assert_eq!(regs[IER], 0);
    }

    #[test]
    fn getchar_returns_none_without_data() {
        let regs = [0u8; 8];
        let uart = Uart16550::new(regs.as_ptr() as usize);
        assert_eq!(uart.getchar(), None);
    }

    #[test]
    fn getchar_reads_rbr_when_ready() {
        let mut regs = [0u8; 8];
        regs[LSR] = LSR_DR;
        regs[RBR] = b'Z';
        let uart = Uart16550::new(regs.as_mut_ptr() as usize);
        assert_eq!(uart.getchar(), Some(b'Z'));
    }
}
