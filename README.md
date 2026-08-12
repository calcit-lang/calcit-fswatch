## Calcit binding for fswatch

> internally it calls [Rust notify](https://github.com/notify-rs/notify) to watch the folder.

API 设计: https://github.com/calcit-lang/calcit_runner.rs/discussions/116 .

### Usages

APIs:

```cirru
fswatch.core/fswatch!
  {}
    :path |folder/
    :duration 200
  fn (event)
    println event
```

Install through `caps`, then compile and provide the platform dynamic library with `./build.sh`.
The Rust dependency on `cirru_edn` must stay compatible with the Calcit runtime
used by the consumer; CI builds and smoke-tests this boundary with the version
declared in `deps.cirru`.

The project keeps one canonical `calcit.cirru` snapshot. Validate it and build
the native library with:

```sh
caps --ci
cr calcit.cirru --check-only
./build.sh
```

Not all events from fswatch are exposed, currently only:

- `:wrote`
- `:created`
- `:removed`
- `:renamed`

a demo of event data:

```cirru
{}
  :type :wrote
  :path |folder/demo.cirru
```

### Workflow

https://github.com/calcit-lang/dylib-workflow

### License

MIT
