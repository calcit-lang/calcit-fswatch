# 升级共享 native FFI 0.1.2 / Upgrade shared native FFI 0.1.2

## 中文

- 升级至 `calcit_native_ffi 0.1.2`，同步共享 raw ABI contract。
- async、resource 与 buffer protocol 仍保持 v1；watcher registry、取消状态与 backpressure 行为不变。
- Rust 集成测试重新验证真实文件事件、callback enqueue 与取消后 COMPLETE；Calcit 0.13.58 trace 验证 async task token 和 cancellable Stream 配置。

## English

- Upgrade to `calcit_native_ffi 0.1.2` and synchronize the shared raw ABI contracts.
- Keep async, resource, and buffer protocols at v1; watcher registry, cancellation state, and backpressure behavior are unchanged.
- Revalidate real file events, callback enqueue, and post-cancel COMPLETE in Rust integration tests; verify the async task token and cancellable Stream configuration in a Calcit 0.13.58 trace.
