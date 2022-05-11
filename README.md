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

Install to `~/.config/calcit/modules/`, compile and provide `*.{dylib,so}` file with `./build.sh`.

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
