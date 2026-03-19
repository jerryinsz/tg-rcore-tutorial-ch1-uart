//! # ch1-uart2：S-mode 下通过 NS16550A 串口驱动输出 "hello world"
//!
//! 本程序演示在 `-bios none` 模式下如何：
//! 1. 由 `m_entry.asm` 中的 M-mode 启动代码完成硬件初始化并切换到 S-mode；
//! 2. 在 S-mode 内核中直接初始化 NS16550A UART 硬件（MMIO）；
//! 3. 通过串口发送 "hello world\n"，不经过任何 SBI 调用。
//!
//! ## 执行流程
//!
//! ```text
//! QEMU (-bios none)
//!   └─ 0x80000000  _m_start  [M-mode, m_entry.asm]
//!        ├─ 配置 PMP / mstatus / medeleg / mideleg
//!        └─ mret → 0x80200000  _start  [S-mode]
//!             ├─ 设置 S-mode 栈指针
//!             └─ call rust_main
//!                  ├─ uart::init()   初始化 NS16550A
//!                  ├─ uart::puts()   发送 "hello world\n"
//!                  └─ shutdown()     写 VIRT_TEST 退出 QEMU
//! ```

#![no_std]
#![no_main]
#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]
#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]

mod uart;

// QEMU virt 平台退出控制寄存器（写入特定值可退出 QEMU）
const VIRT_TEST: usize = 0x10_0000;
const EXIT_SUCCESS: u32 = 0x5555;
const EXIT_FAILURE: u32 = 0x3333;

/// 通过写入 QEMU virt test 寄存器退出模拟器。
fn shutdown(failure: bool) -> ! {
    let code = if failure { EXIT_FAILURE } else { EXIT_SUCCESS };
    unsafe {
        (VIRT_TEST as *mut u32).write_volatile(code);
    }
    loop {}
}

// 嵌入 M-mode 启动代码（仅 RISC-V64 目标）。
// `-bios none` 时 QEMU 从 0x80000000 开始执行，此处放置 M-mode 初始化代码。
#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(include_str!("m_entry.asm"));

/// S-mode 内核入口（链接到 0x80200000）。
///
/// 裸函数：设置 S-mode 栈，然后跳转到 `rust_main`。
#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    /// S-mode 内核栈大小：4 KiB
    const STACK_SIZE: usize = 4096;

    #[unsafe(link_section = ".bss.uninit")]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    core::arch::naked_asm!(
        "la  sp, {stack} + {stack_size}", // sp 指向栈顶
        "call {main}",                    // 调用 rust_main
        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
    )
}

/// S-mode 主函数：初始化串口并输出 "hello world"。
extern "C" fn rust_main() -> ! {
    // 初始化 NS16550A UART（设置波特率、帧格式、FIFO）
    uart::init();
    // 通过串口发送字符串（不经过 SBI）
    uart::puts(b"hello world\n");
    shutdown(false)
}

/// panic 处理：以异常状态退出 QEMU。
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    shutdown(true)
}

/// 非 RISC-V64 平台占位（满足 `cargo check` / `cargo publish` 需求）。
#[cfg(not(target_arch = "riscv64"))]
mod stub {
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 { 0 }

    #[unsafe(no_mangle)]
    pub extern "C" fn __libc_start_main() -> i32 { 0 }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_eh_personality() {}
}
