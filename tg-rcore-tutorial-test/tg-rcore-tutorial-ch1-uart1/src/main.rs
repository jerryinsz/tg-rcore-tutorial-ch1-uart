#![no_std]
#![no_main]
#![cfg_attr(target_arch = "riscv64", deny(warnings))]
#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]

mod uart;

// QEMU virt 的 test device，用于在 -bios none 下退出模拟器
const VIRT_TEST: usize = 0x10_0000;
const EXIT_SUCCESS: u32 = 0x5555;
const EXIT_FAILURE: u32 = 0x3333;

// 不依赖 SBI，直接写 test device 触发 QEMU 退出
fn shutdown(failure: bool) -> ! {
    let code = if failure { EXIT_FAILURE } else { EXIT_SUCCESS };
    unsafe {
        (VIRT_TEST as *mut u32).write_volatile(code);
    }
    loop {}
}

#[cfg(target_arch = "riscv64")]
// 内联 M 态启动桥：完成 M->S 切换后进入 _start
core::arch::global_asm!(include_str!("m_entry.asm"));

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    // S 态最小栈，供 rust_main 使用
    const STACK_SIZE: usize = 4096;
    #[unsafe(link_section = ".bss.uninit")]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];
    core::arch::naked_asm!(
        "la sp, {stack} + {stack_size}",
        "j  {main}",
        stack_size = const STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
    )
}

extern "C" fn rust_main() -> ! {
    // 在 S 态初始化串口并直接 MMIO 输出，不走 SBI console
    uart::init();
    uart::puts(b"Hello, world!\n");
    shutdown(false)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    shutdown(true)
}

#[cfg(not(target_arch = "riscv64"))]
mod stub {
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        0
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn __libc_start_main() -> i32 {
        0
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_eh_personality() {}
}
