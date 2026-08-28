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

Install through `caps`, then compile and provide the platform dynamic library with `./build.sh`.
The native library uses Calcit's C-safe cancellable async Stream protocol v1 and
requires Calcit 0.13.58 or newer. Shared descriptors, validation, Cirru EDN
transport, host calls, and backpressure policy come from
[`calcit_native_ffi`](https://github.com/calcit-lang/calcit-native-ffi).
The module tracks `calcit_native_ffi 0.1.3`; async, resource, and buffer
protocols remain at v1.
The watcher registry, thread, and cancellation state remain module-owned, while
the shared crate owns deadline-aware retry and polls the module predicate at
most every 10ms. Legacy Rust callback ABI symbols are no longer exported.

原生库要求 Calcit 0.13.58 或更新版本。Descriptor、validation、Cirru EDN
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

The watcher declares serialized callback delivery with coalescing allowed. The
runtime can cancel it during shutdown; cancellation drops the native watcher and
publishes an explicit completion event.

a demo of event data:

```cirru
FswatchEvent :type :modify :path |folder/demo.cirru :extra |Data(Content)
```

### Workflow

https://github.com/calcit-lang/dylib-workflow

### License

MIT
