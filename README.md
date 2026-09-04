## Calcit binding for fswatch

> internally it calls [Rust notify](https://github.com/notify-rs/notify) to watch the folder.

API 设计: https://github.com/calcit-lang/calcit_runner.rs/discussions/116 .

### Usages

APIs:

```cirru
fswatch.core/fswatch!
  fswatch.core/FswatchOptions :path |folder/ :duration 200
  fn (event)
    println event.:type event.:path
```

`fswatch!` returns a typed `FfiTask`. Keep the task when its lifetime must be
controlled explicitly, then call `.cancel` or `.cancel-with` during component
or server shutdown.

`fswatch!` 返回类型化的 `FfiTask`。需要显式控制监听生命周期时应保留该任务，
并在组件或服务停止时调用 `.cancel` 或 `.cancel-with`。

Install through `caps`, then compile and provide the platform dynamic library with `./build.sh`.
The native library uses Calcit's C-safe cancellable async Stream protocol v1 and
requires Calcit 0.13.77 or newer. Shared descriptors, validation, Cirru EDN
transport, host calls, and backpressure policy come from
[`calcit_native_ffi`](https://github.com/calcit-lang/calcit-native-ffi).
The module tracks `calcit_native_ffi 0.1.3`; async, resource, and buffer
protocols remain at v1.
The watcher registry, thread, and cancellation state remain module-owned, while
the shared crate owns deadline-aware retry and polls the module predicate at
most every 10ms. Legacy Rust callback ABI symbols are no longer exported.

原生库要求 Calcit 0.13.77 或更新版本。Descriptor、validation、Cirru EDN
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
FswatchEvent :type :modify :path |folder/demo.cirru :extra |Data(Content)
```

### Workflow

https://github.com/calcit-lang/dylib-workflow

### License

MIT
