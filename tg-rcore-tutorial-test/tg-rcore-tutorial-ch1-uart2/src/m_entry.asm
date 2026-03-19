# `-bios none` 场景下的 M 态入口代码
# QEMU virt 平台在 -bios none 时从 0x80000000 开始执行，处于 M 态
#
# 本文件完成：
#   1. 搭建 M 态专用栈
#   2. 配置 mstatus.MPP = 01（返回到 S 态）
#   3. 设置 mepc 指向 S 态内核入口 _start
#   4. 配置 PMP：允许 S 态访问全部物理地址
#   5. 允许 S 态读取时钟计数器
#   6. mret 切换到 S 态，开始执行内核

    .section .text.m_entry
    .globl _m_start
_m_start:
    # 1) 建立 M 态专用栈
    la   sp, m_stack_top
    csrw mscratch, sp

    # 2) mstatus: MPP=01 (S-mode), MPIE=0
    li   t0, (1 << 11)
    csrw mstatus, t0

    # 3) 返回地址指向 S 态内核入口
    la   t0, _start
    csrw mepc, t0

    # 4) 中断/异常委托给 S 态（本例中 S 态不调用 SBI，无需处理 ecall，
    #    但将所有中断/异常委托给 S 态，以便将来扩展）
    li   t0, 0xffff
    csrw mideleg, t0
    li   t0, 0xffff
    csrw medeleg, t0

    # 5) PMP：允许 S 态访问全部物理地址（TOR 模式 + RWX，addr = -1）
    li   t0, -1
    csrw pmpaddr0, t0
    li   t0, 0x0f
    csrw pmpcfg0, t0

    # 6) 允许 S 态读取计数器
    li   t0, -1
    csrw mcounteren, t0

    # 7) mret 跳转到 S 态内核
    mret

    # M 态专用栈（4 KiB）
    .section .bss.m_stack
    .align 12
    .space 4096
    .globl m_stack_top
m_stack_top:
