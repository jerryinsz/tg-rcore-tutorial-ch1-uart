    .section .text.m_entry
    .globl _m_start
_m_start:
    # 建立 M 态栈
    la   sp, m_stack_top
    csrw mscratch, sp
    # 设置 mstatus.MPP=S，mret 后切到 S 态
    li   t0, (1 << 11)
    csrw mstatus, t0
    # 返回入口指向 S 态 _start
    la   t0, _start
    csrw mepc, t0
    # 委托中断与异常给 S 态
    li   t0, 0xffff
    csrw mideleg, t0
    li   t0, 0xffff
    csrw medeleg, t0
    # 开放 PMP，允许 S 态访问物理内存
    li   t0, -1
    csrw pmpaddr0, t0
    li   t0, 0x0f
    csrw pmpcfg0, t0
    # 允许 S 态读取计数器
    li   t0, -1
    csrw mcounteren, t0
    # 切换到 S 态执行
    mret

    # M 态栈空间（4KiB）
    .section .bss.m_stack
    .align 12
    .space 4096
    .globl m_stack_top
m_stack_top:
