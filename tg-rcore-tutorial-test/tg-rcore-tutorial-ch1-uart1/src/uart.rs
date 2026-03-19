// QEMU virt 平台 NS16550A UART 基址
const UART_BASE: usize = 0x1000_0000;
// DLAB=0 时偏移 0 读为 RBR
const RBR: usize = UART_BASE + 0;
// DLAB=0 时偏移 0 写为 THR
const THR: usize = UART_BASE + 0;
// DLAB=1 时偏移 0 写为 DLL
const DLL: usize = UART_BASE + 0;
const IER: usize = UART_BASE + 1;
const DLM: usize = UART_BASE + 1;
const FCR: usize = UART_BASE + 2;
const LCR: usize = UART_BASE + 3;
const MCR: usize = UART_BASE + 4;
const LSR: usize = UART_BASE + 5;
// LSR bit5=1 表示发送保持寄存器空
const LSR_THRE: u8 = 1 << 5;

#[inline]
unsafe fn write_reg(addr: usize, val: u8) {
    unsafe { (addr as *mut u8).write_volatile(val) };
}

#[inline]
unsafe fn read_reg(addr: usize) -> u8 {
    unsafe { (addr as *const u8).read_volatile() }
}

pub fn init() {
    unsafe {
        // 关闭中断，进入 DLAB 配置波特率
        write_reg(IER, 0x00);
        write_reg(LCR, 0x80);
        // 3_686_400 / (16 * 115200) = 2
        write_reg(DLL, 0x02);
        write_reg(DLM, 0x00);
        // 8N1，退出 DLAB
        write_reg(LCR, 0x03);
        // 启用并清空 FIFO
        write_reg(FCR, 0xC7);
        // 使能 DTR/RTS
        write_reg(MCR, 0x0B);
        // 读取一次状态寄存器，清理可能的挂起状态
        let _ = read_reg(LSR);
        let _ = read_reg(RBR);
    }
}

pub fn putchar(c: u8) {
    unsafe {
        // 忙等待直到 THR 可写
        while read_reg(LSR) & LSR_THRE == 0 {}
        write_reg(THR, c);
    }
}

pub fn puts(s: &[u8]) {
    for &c in s {
        putchar(c);
    }
}
