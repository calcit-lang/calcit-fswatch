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
requires Calcit 0.13.52 or newer. Legacy Rust callback ABI symbols are no longer
exported.

The project keeps one canonical `calcit.cirru` snapshot. Validate it and build
the native library with:

```sh
caps --ci
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
