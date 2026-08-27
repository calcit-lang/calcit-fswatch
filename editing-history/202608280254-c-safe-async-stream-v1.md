# C-safe async Stream v1 / C 安全异步流 v1

- Migrated filesystem watching from Rust callback trait objects to C-safe async protocol v1.
- 将文件监听从 Rust callback trait object 迁移到 C 安全 async protocol v1。
- Configured a cancellable serialized Stream with coalescing allowed, bounded cancellation latency, explicit completion, and panic/error containment.
- 配置可取消、串行、允许合并的 Stream，并提供有界取消延迟、显式完成及 panic/error 隔离。
- Added native watcher cancellation tests, event-shape coverage, symbol auditing, and a real traced Calcit smoke.
- 增加 native watcher 取消测试、事件结构覆盖、符号审计与真实 traced Calcit smoke。
- Added typed `FswatchOptions` and `FswatchEvent` Structs and brought every local definition to full schema coverage; the native decoder temporarily accepts the legacy options map.
- 增加类型化的 `FswatchOptions` 与 `FswatchEvent` Struct，使所有本地定义具备完整 schema；native decoder 暂时兼容旧 options map。
- Upgraded to Calcit 0.13.52, strict macro schemas, and `setup-calcit@v1.3.0`.
- 升级到 Calcit 0.13.52、严格 macro schema 与 `setup-calcit@v1.3.0`。
