# tg-rcore-tutorial-uart

[![Crates.io](https://img.shields.io/crates/v/tg-rcore-tutorial-uart.svg)](https://crates.io/crates/tg-rcore-tutorial-uart)
[![Documentation](https://docs.rs/tg-rcore-tutorial-uart/badge.svg)](https://docs.rs/tg-rcore-tutorial-uart)

`tg-rcore-tutorial-uart` 是一个面向 rCore 教学实验的极简 UART16550 串口驱动组件，运行于 `no_std` 环境，可用于 RISC-V S 态内核直接通过 MMIO 输出与读取字符。

## 目标

- 提供可复用的串口收发组件 crate
- 适配 QEMU `virt` 平台 UART0（基址 `0x10000000`）
- 支持 `cargo test` 的单元测试验证
- 满足发布到 crates.io 的基础元信息要求

## 架构

```mermaid
flowchart LR
    K[S-Mode Kernel] --> U[Uart16550]
    U --> M[MMIO UART0 0x10000000]
    M --> T[Terminal via QEMU -nographic]
```

## 使用示例

```rust
use tg_rcore_tutorial_uart::Uart16550;

let uart = Uart16550::qemu_virt();
uart.init();
uart.putstr("Hello, UART!\n");
let _ = uart.getchar();
```

## 对外接口

- `init()`：初始化串口驱动的最小状态
- `putchar(u8)`：发送单字节
- `putstr(&str)`：发送字符串
- `getchar() -> Option<u8>`：尝试读取一个字节

## 寄存器语义（QEMU virt UART0）

| 寄存器 | 偏移 | 作用 |
|---|---:|---|
| `RBR` | `0` | 接收缓冲读取 |
| `THR` | `0` | 发送保持写入 |
| `IER` | `1` | 中断使能 |
| `LSR` | `5` | 线路状态 |

关键位：

- `LSR[0]`：接收数据就绪（DR）
- `LSR[5]`：发送寄存器可写（THRE）

## 测试

```bash
cargo test
```

## License

GPL-3.0
