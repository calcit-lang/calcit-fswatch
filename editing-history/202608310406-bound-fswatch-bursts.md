# 有界文件事件突发与确定终止 / Bound filesystem-event bursts and deterministic termination

## 中文

- 将 `notify` 回调与 Calcit host 之间的无界 channel 改为双重上限入口队列：最多 256 个已编码事件或 1 MiB。
- 保持事件与 rename 路径对的源顺序，不再声明 host coalescing；一个事件批次整体接收或整体拒绝，不产生部分 rename。
- 入口溢出、watcher 错误和 host 背压拒绝都会清空待处理入口并以一次 `Fail` 终止；取消优先并以一次 `Complete` 终止，调用方应在失败后 rescan 再重启。
- 增加 10,000 次突发、有界高水位、批次原子性、背压可取消、事件映射和 exactly-once terminal 测试；Linux 运行真实 create/modify/rename/remove 生命周期测试。macOS FSEvents 对临时 Git worktree 不稳定，因此该项只在 macOS 跳过。
- 将最低 Calcit 版本升级到 `0.13.68`，公开 schema 改为返回 typed `FfiTask`，并使用当前 symbol-key Snapshot 格式及中英双语文档。

## English

- Replace the unbounded channel between the `notify` callback and the Calcit host with a dual-bounded ingress: at most 256 encoded events or 1 MiB.
- Preserve source order for events and rename path pairs without opting into host coalescing; admit or reject one event batch atomically so a rename cannot be partially retained.
- Clear pending ingress and terminate with one `Fail` on overflow, watcher errors, or host backpressure rejection; cancellation takes precedence and terminates with one `Complete`. Consumers must rescan before restarting after failure.
- Add 10,000-item burst, bounded high-water, batch atomicity, cancellable backpressure, event mapping, and exactly-once terminal tests. Linux runs a real create/modify/rename/remove lifecycle test; it is skipped only on macOS because FSEvents is unreliable for temporary Git worktrees.
- Raise the minimum Calcit version to `0.13.68`, expose a typed `FfiTask` return schema, and adopt the current symbol-key Snapshot format with bilingual documentation.
