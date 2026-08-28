# 使用共享 native FFI crate / Adopt the shared native FFI crate

## 中文

- 使用 `calcit_native_ffi 0.1.1` 统一 async v1 descriptor、校验、Cirru EDN transport、host call 和 backpressure policy。
- 保留 watcher registry、线程、取消状态，以及队列满时可取消的 emit retry loop 在模块侧。
- 增加背压取消回归测试，并将 Calcit 与依赖/类型门禁更新到 0.13.57 严格模式。

## English

- Use `calcit_native_ffi 0.1.1` for async-v1 descriptors, validation, Cirru EDN transport, host calls, and backpressure policy.
- Keep the watcher registry, thread, cancellation state, and queue-full cancellation-aware emit retry loop module-owned.
- Add a backpressure-cancellation regression and update the Calcit, dependency, and type gates to strict 0.13.57 behavior.
