//! 构建脚本：为 RISC-V64 目标生成链接脚本。
//!
//! 内存布局：
//! - 0x80000000：M-mode 入口及陷阱处理（m_entry.asm 提供）
//! - 0x80200000：S-mode 内核入口（_start）

fn main() {
    use std::{env, fs, path::PathBuf};

    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "riscv64" {
        let ld = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("linker.ld");
        fs::write(&ld, LINKER_SCRIPT).unwrap();
        println!("cargo:rustc-link-arg=-T{}", ld.display());
    }
}

/// 链接脚本：
///
/// ```text
/// 0x80000000  M-mode 区域
///   .text.m_entry  M-mode 启动代码（_m_start）
///   .text.m_trap   M-mode 陷阱处理
///   .bss.m_stack   M-mode 专用栈（16 KiB）
///   .bss.m_data    M-mode 数据
///
/// 0x80200000  S-mode 区域
///   .text          代码段（含 .text.entry 入口 _start）
///   .rodata        只读数据
///   .data          可读写数据
///   .bss           未初始化数据（含 S-mode 栈）
/// ```
const LINKER_SCRIPT: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_m_start)

M_BASE_ADDRESS = 0x80000000;
S_BASE_ADDRESS = 0x80200000;

SECTIONS {
    . = M_BASE_ADDRESS;
    .text.m_entry : { *(.text.m_entry) }
    .text.m_trap  : { *(.text.m_trap)  }
    .bss.m_stack  : { *(.bss.m_stack)  }
    .bss.m_data   : { *(.bss.m_data)   }

    . = S_BASE_ADDRESS;
    .text   : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data   : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss    : {
        *(.bss.uninit)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
}
";
