# 维护指南 / Maintainer guide

## 中文

- 修改 `calcit.cirru` 前先运行 `calcit docs agents --full`，并只使用 `calcit edit` / `calcit tree` 修改 Snapshot。
- C-safe async descriptor、校验、Cirru EDN transport、host call 与通用 backpressure policy 由 `calcit_native_ffi` 维护。
- watcher registry、线程与取消状态属于本模块；通过 predicate 接入共享 crate 的 cancellation-aware、deadline-aware retry，不要复制重试循环。
- 提交前运行 Rust fmt/test/strict Clippy、`caps --strict --ci`、Calcit quality/dynamic-method gates、symbol audit 和真实 watcher smoke。

## English

- Run `calcit docs agents --full` before changing `calcit.cirru`, and modify the Snapshot only through `calcit edit` / `calcit tree`.
- `calcit_native_ffi` owns C-safe async descriptors, validation, Cirru EDN transport, host calls, and the general backpressure policy.
- The watcher registry, thread, and cancellation state remain module-owned. Connect them by predicate to the shared crate's cancellation-aware, deadline-aware retry instead of copying a retry loop.
- Before committing, run Rust fmt/tests/strict Clippy, `caps --strict --ci`, Calcit quality/dynamic-method gates, symbol audit, and a real watcher smoke.
