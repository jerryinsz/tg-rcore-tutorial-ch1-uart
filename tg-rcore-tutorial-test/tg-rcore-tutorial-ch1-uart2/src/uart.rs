//! NS16550A UART 驱动
//!
//! QEMU virt 平台将 NS16550A 兼容 UART 映射到物理地址 0x1000_0000。
//! 本驱动在 S-mode 下通过 MMIO 直接访问硬件寄存器，无需 SBI 调用。
//!
//! ## NS16550A 寄存器（以字节方式访问，基址 = 0x1000_0000）
//!
//! | 偏移 | DLAB=0 读  | DLAB=0 写  | DLAB=1     |
//! |------|-----------|-----------|------------|
//! | 0    | RBR（接收）| THR（发送）| DLL（波特率低字节）|
//! | 1    | IER（中断）| IER        | DLM（波特率高字节）|
//! | 2    | IIR（中断状态）| FCR（FIFO）|          |
//! | 3    | LCR（线路控制）| LCR     |            |
//! | 4    | MCR（调制解调器）| MCR  |            |
//! | 5    | LSR（线路状态）| —       |            |
//!
//! ## 初始化步骤
//!
//! 1. 禁用中断（IER = 0x00）
//! 2. 进入波特率设置模式（LCR.DLAB = 1）
//! 3. 设置波特率除数（QEMU virt 时钟 3686400 Hz，115200 波特，除数 = 2）
//! 4. 设置帧格式（LCR = 8N1，DLAB = 0）
//! 5. 使能并复位 FIFO（FCR = 0xC7）
//! 6. 使能 DTR/RTS（MCR = 0x0B）

/// UART 基址（QEMU virt 平台）
const UART_BASE: usize = 0x1000_0000;

// ----- 寄存器偏移量 -----
const RBR: usize = UART_BASE + 0; // Receive Buffer Register  (DLAB=0, read)
const THR: usize = UART_BASE + 0; // Transmit Holding Register (DLAB=0, write)
const DLL: usize = UART_BASE + 0; // Divisor Latch Low  (DLAB=1)
const IER: usize = UART_BASE + 1; // Interrupt Enable Register (DLAB=0)
const DLM: usize = UART_BASE + 1; // Divisor Latch High (DLAB=1)
const FCR: usize = UART_BASE + 2; // FIFO Control Register (write)
const LCR: usize = UART_BASE + 3; // Line Control Register
const MCR: usize = UART_BASE + 4; // Modem Control Register
const LSR: usize = UART_BASE + 5; // Line Status Register

// ----- LSR 位定义 -----
/// 发送保持寄存器空（可写入下一字节）
const LSR_THRE: u8 = 1 << 5;

// ----- 辅助函数 -----

#[inline]
unsafe fn write_reg(addr: usize, val: u8) {
    unsafe { (addr as *mut u8).write_volatile(val) };
}

#[inline]
unsafe fn read_reg(addr: usize) -> u8 {
    unsafe { (addr as *const u8).read_volatile() }
}

/// 初始化 NS16550A UART。
///
/// 配置为：115200 波特，8 位数据，无校验，1 位停止位（8N1），
/// 使能 16 字节发送/接收 FIFO。
///
/// # Safety
/// 必须在访问 UART 之前调用一次，且调用者须保证 UART_BASE 已映射可访问。
pub fn init() {
    unsafe {
        // 1. 禁用所有中断
        write_reg(IER, 0x00);

        // 2. 进入 DLAB 模式以设置波特率
        write_reg(LCR, 0x80);

        // 3. 设置波特率除数
        //    QEMU virt UART 输入时钟 = 3_686_400 Hz
        //    波特率 = 115200 → 除数 = 3_686_400 / (16 * 115200) = 2
        write_reg(DLL, 0x02); // 除数低字节
        write_reg(DLM, 0x00); // 除数高字节

        // 4. 设置帧格式：8 位数据，1 位停止位，无校验（LCR = 0x03）
        //    同时退出 DLAB 模式（位7 = 0）
        write_reg(LCR, 0x03);

        // 5. 使能 FIFO，复位收发 FIFO，触发阈值 14 字节
        //    FCR[0]=1 启用FIFO，FCR[1]=1 清空RX FIFO，FCR[2]=1 清空TX FIFO，
        //    FCR[7:6]=11 设置触发阈值为 14 字节
        write_reg(FCR, 0xC7);

        // 6. 使能 DTR 和 RTS（MCR = 0x0B）
        write_reg(MCR, 0x0B);

        // 7. 读取 LSR/IIR 清除可能的挂起状态（丢弃返回值）
        let _ = read_reg(LSR);
        let _ = read_reg(RBR);
    }
}

/// 向 UART 发送一个字节。
///
/// 忙等待直到发送保持寄存器为空（LSR.THRE = 1），再写入 THR。
pub fn putchar(c: u8) {
    unsafe {
        while read_reg(LSR) & LSR_THRE == 0 {}
        write_reg(THR, c);
    }
}

/// 向 UART 发送一个字节串。
pub fn puts(s: &[u8]) {
    for &c in s {
        putchar(c);
    }
}
