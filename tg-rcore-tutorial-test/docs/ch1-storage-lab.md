# ch1-storage 实验指导文档

## 1. 实验目标

- 扩展 `tg-rcore-tutorial-ch1`，形成 `tg-rcore-tutorial-ch1-storage` 极简内核 crate
- 支持在 S-Mode 下完成最小磁盘块读写
- 形成可复现实验流程与验证脚本

## 2. 技术路径

```mermaid
flowchart LR
    K[S-Mode Kernel] --> V[VirtIOBlk Driver]
    V --> M[MMIO 0x10001000]
    M --> D[QEMU fs.img]
```

## 3. 中断机制设计点

- 块设备读写完成后，读取 VirtIO MMIO `interrupt status`
- 向 `interrupt ack` 写入同样值完成应答
- 本实验把“中断状态读取与应答”纳入最小验证路径，便于后续扩展到完整外部中断分发

## 4. 实验步骤

### 4.1 构建磁盘镜像

```bash
cd tg-rcore-tutorial-ch1-storage
dd if=/dev/zero of=fs.img bs=1M count=16 status=none
```

### 4.2 运行内核

```bash
cargo run
```

### 4.3 预期输出

```text
ch1-storage: start
irq_after_write=...
irq_after_read=...
STORAGE-OK
```

## 5. 验证命令

```bash
cd tg-rcore-tutorial-ch1-storage
bash test.sh
```

## 6. 关键代码入口

- `tg-rcore-tutorial-ch1-storage/src/main.rs`
- `tg-rcore-tutorial-ch1-storage/.cargo/config.toml`

## 7. 可扩展方向

- 增加多块读写与随机块测试
- 接入文件系统层（如 easy-fs）验证文件语义
- 引入完整外部中断分发与设备中断处理路径
