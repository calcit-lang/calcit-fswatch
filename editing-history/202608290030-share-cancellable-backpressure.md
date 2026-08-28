# 复用共享可取消背压 / Reuse shared cancellable backpressure

## 中文

- 升级已发布的 `calcit_native_ffi 0.1.3`。
- 将项目声明的 Calcit 最低版本与实际验证版本 `0.13.58` 对齐。
- 删除模块内重复的 queue-full retry loop，文件事件统一使用共享 `publish_emit_until`。
- watcher 继续拥有 registry、线程与取消状态，并通过 predicate 接入共享 10ms 取消轮询和 5 秒 deadline。

## English

- Upgrade to the published `calcit_native_ffi 0.1.3`.
- Align the declared minimum Calcit version with the tested `0.13.58` release.
- Remove the duplicated queue-full retry loop and publish filesystem events through shared `publish_emit_until`.
- Keep the watcher registry, thread, and cancellation state module-owned, connecting them by predicate to shared 10ms cancellation polling and the five-second deadline.
