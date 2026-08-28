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
requires Calcit 0.13.57 or newer. Shared descriptors, validation, Cirru EDN
transport, host calls, and backpressure policy come from
[`calcit_native_ffi`](https://github.com/calcit-lang/calcit-native-ffi).
The watcher registry, thread, cancellation state, and cancellation-aware emit
retry loop remain module-owned. Legacy Rust callback ABI symbols are no longer exported.

原生库要求 Calcit 0.13.57 或更新版本。Descriptor、validation、Cirru EDN
transport、host call 与 backpressure policy 由
[`calcit_native_ffi`](https://github.com/calcit-lang/calcit-native-ffi) 维护；
watcher registry、线程、取消状态，以及 emit 重试期间的取消检查继续由模块维护。

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
