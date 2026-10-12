## Calcit binding for fswatch

> internally it calls [Rust notify](https://github.com/notify-rs/notify) to watch the folder.

API 设计: https://github.com/calcit-lang/calcit_runner.rs/discussions/116 .

### Usages

APIs:

```cirru
fswatch.core/fswatch! (fswatch.core/FswatchOptions :path |folder/ :duration 200)
  fn (event)
    hint-fn $ {}
      :args $ [] 'fswatch.core/FswatchEvent
      :return 'Unit
    println (:type event) (:path event)
```

`fswatch!` returns a typed `FfiTask`. Keep the task when its lifetime must be
controlled explicitly, then call `.cancel!` or `.cancel-with!` during component
or server shutdown.

`fswatch!` 返回类型化的 `FfiTask`。需要显式控制监听生命周期时应保留该任务，
并在组件或服务停止时调用 `.cancel!` 或 `.cancel-with!`。

Install through `caps`, then compile and provide the platform dynamic library with `./build.sh`.
The native library uses Calcit's C-safe cancellable async Stream protocol v1 and
uses the project's pinned Calcit 0.29.0. Shared descriptors, validation, Cirru EDN
transport, host calls, and backpressure policy come from
[`calcit_native_ffi`](https://github.com/calcit-lang/calcit-native-ffi).
The module tracks `calcit_native_ffi 0.1.3`; async, resource, and buffer
protocols remain at v1.
The watcher registry, thread, and cancellation state remain module-owned, while
the shared crate owns deadline-aware retry and polls the module predicate at
most every 10ms. Legacy Rust callback ABI symbols are no longer exported.

项目使用正式 Calcit 0.29.0。Descriptor、validation、Cirru EDN
transport、host call 与 backpressure policy 由
[`calcit_native_ffi`](https://github.com/calcit-lang/calcit-native-ffi) 维护；
模块当前使用 `calcit_native_ffi 0.1.3`，async、resource 与 buffer protocol
均继续保持 v1；
watcher registry、线程与取消状态继续由模块维护，共享 crate 负责有 deadline 的
重试，并最长每 10ms 检查一次模块提供的取消 predicate。

The project keeps one canonical `calcit.cirru` snapshot. Validate it and build
the native library with:

```sh
caps --strict --ci
calcit calcit.cirru --check-only
./build.sh
```

Not all events from fswatch are exposed, currently only:

- `:modify`
- `:create`
- `:remove`
- `:rename`

The watcher preserves source order and does not opt into host-side coalescing:
rename pairs and nearby remove/create transitions are never silently replaced.
Its native ingress is bounded to 256 encoded events and 1 MiB. A burst beyond
either limit clears retained ingress and terminates the task with one explicit
failure; the consumer must rescan the watched state before restarting. Normal
cancellation drops the native watcher and publishes exactly one completion.

监听器保持源事件顺序，并且不启用 host 侧合并：rename 路径对及相邻的
remove/create 转换不会被静默替换。原生入口队列上限为 256 个编码事件和
1 MiB；突发流量超过任一限制时会清空尚未消费的入口事件，并以一次显式失败
终止任务。消费者必须重新扫描被监听状态后再启动新任务。正常取消会释放原生
watcher，并且只发布一次完成事件。

a demo of event data:

```cirru
fswatch.core/FswatchEvent :type :modify :path |folder/demo.cirru :extra |Data-content
```

### Workflow

https://github.com/calcit-lang/dylib-workflow

本仓库使用正式 Calcit 0.29.0，默认入口显式声明 native target，空 reload
返回 `&unit`。原生任务句柄通过 `ffi:task` 包装为真正的 FfiTask；原生事件在
适配边界经 `decode-map-as` 恢复并校验 FswatchEvent 的类型与字段。
公开合同、watcher 的事件顺序、取消和 backpressure 行为，以及 Rust 依赖与
协议版本保持不变。
没有前端资源，不添加 COS 或额外验证脚本。

CI 使用严格入口和全部项目命名空间检查、weak-types dynamic-method 视图、
Rust 测试、异步 FFI 符号审计、文档执行和五秒有界 watcher smoke，并检查
工具链匹配。不再使用已弃用的 quality/check-public/dynamic-methods 入口。
smoke 只证明异步导出实际加载；真实文件事件和取消完成
由已有 Rust 测试验收。该事件测试在 macOS 按原策略跳过，以 Linux CI 为准。

Action 使用正式标签，标签可移动；只读权限和禁用 checkout 凭据持久化
不保证供应链不可变。不新增迁移规则、类型强转或测试框架。

### License

MIT
