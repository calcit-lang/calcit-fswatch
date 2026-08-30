# 稳定 Linux 事件生命周期回归 / Stabilize the Linux event lifecycle regression

## 中文

- Linux `notify` 在快速连续 create/write/rename/remove 时可以在内核投递 modify 前完成 rename，因此测试需要逐阶段等待对应路径事件，而不是一次执行全部操作后等待类型集合。
- 将数据修改映射从仅 `Data(Content)` 扩展为全部 `Modify(Data(_))`，覆盖 Linux 常见的 `Data(Any)`，同时继续排除 metadata-only 修改。
- 真实测试现在按唯一文件路径依次确认 create、modify、rename、remove，避免旧的 readiness 事件误满足断言。

## English

- On Linux, a rapid create/write/rename/remove sequence may finish the rename before the kernel delivers a modify event, so the test now waits after each lifecycle operation instead of issuing all operations before checking event types.
- Broaden data-modification mapping from only `Data(Content)` to every `Modify(Data(_))`, covering Linux's common `Data(Any)` while continuing to exclude metadata-only changes.
- The real test now confirms create, modify, rename, and remove sequentially for one unique file path so earlier readiness events cannot satisfy lifecycle assertions.
