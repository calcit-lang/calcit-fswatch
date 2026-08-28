use calcit_native_ffi::{
  BackpressurePolicy, CalcitFfiAsyncHostV1, CalcitFfiAsyncTaskV1, configure_task, copy_async_host, copy_task_descriptor,
  decode_request, publish_complete as publish_async_complete, publish_emit_until as publish_async_emit_until,
  publish_failure as publish_async_failure, status, task_flags, task_kind,
};
use cirru_edn::{Edn, EdnStructView};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::spawn;
use std::time::Duration;

calcit_native_ffi::export_async_abi_v1!();

struct WatchControl {
  cancelled: AtomicBool,
  host: CalcitFfiAsyncHostV1,
  task: CalcitFfiAsyncTaskV1,
}

static WATCH_CONTROLS: OnceLock<Mutex<HashMap<u64, Arc<WatchControl>>>> = OnceLock::new();
static NEXT_WATCH_CONTEXT: AtomicU64 = AtomicU64::new(1);

fn watch_controls() -> &'static Mutex<HashMap<u64, Arc<WatchControl>>> {
  WATCH_CONTROLS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_watch_context() -> u64 {
  loop {
    let id = NEXT_WATCH_CONTEXT.fetch_add(1, Ordering::Relaxed);
    if id != 0 {
      return id;
    }
  }
}

fn parse_options(args: &[Edn]) -> Result<(Arc<str>, Duration), String> {
  let [options] = args else {
    return Err(format!("fswatch expected one options value, got: {args:?}"));
  };
  let get_field = |name: &str| match options {
    Edn::Map(options) => options.get(&Edn::tag(name)),
    Edn::Struct(options) if options.name.as_ref() == "FswatchOptions" => options
      .pairs
      .iter()
      .find(|(field, _)| field.ref_str() == name)
      .map(|(_, value)| value),
    _ => None,
  };
  let path = get_field("path").ok_or("fswatch :path is required")?.read_str()?.to_owned();
  let milliseconds = get_field("duration").ok_or("fswatch :duration is required")?.read_number()?;
  if !milliseconds.is_finite() || milliseconds <= 0.0 || milliseconds > u64::MAX as f64 {
    return Err(format!("fswatch :duration must be a finite positive number, got {milliseconds}"));
  }
  Ok((path, Duration::from_millis(milliseconds as u64)))
}

fn publish_emit(control: &WatchControl, event: Edn) -> i32 {
  publish_async_emit_until(control.host, control.task, vec![event], BackpressurePolicy::default(), || {
    !control.cancelled.load(Ordering::Acquire)
  })
}

fn publish_failure(control: &WatchControl, message: impl Into<String>) -> i32 {
  publish_async_failure(control.host, control.task, message, BackpressurePolicy::default())
}

fn publish_complete(control: &WatchControl) -> i32 {
  publish_async_complete(control.host, control.task, BackpressurePolicy::default())
}

fn new_event(kind: &str, path: &Path, extra: &str) -> Edn {
  let mut event = EdnStructView::new("FswatchEvent");
  event.insert("type", Edn::tag(kind));
  event.insert("path", Edn::str(path.display().to_string()));
  event.insert("extra", Edn::str(extra));
  Edn::Struct(event)
}

fn map_event(event: Event) -> Vec<Edn> {
  let kind = match &event.kind {
    EventKind::Modify(ModifyKind::Data(DataChange::Content)) => "modify",
    EventKind::Modify(ModifyKind::Name(_)) => "rename",
    EventKind::Create(_) => "create",
    EventKind::Remove(_) => "remove",
    _ => return vec![],
  };
  let extra = format!("{:?}", event.kind);
  event.paths.iter().map(|path| new_event(kind, path, &extra)).collect()
}

fn run_watcher(path: Arc<str>, poll_interval: Duration, control: Arc<WatchControl>) -> Result<(), String> {
  let (tx, rx) = channel();
  let config = notify::Config::default().with_poll_interval(poll_interval);
  let mut watcher = RecommendedWatcher::new(tx, config).map_err(|error| format!("failed to create watcher: {error}"))?;
  watcher
    .watch(Path::new(&*path), RecursiveMode::Recursive)
    .map_err(|error| format!("failed to watch path {path}: {error}"))?;

  let cancel_poll = poll_interval.min(Duration::from_millis(100));
  while !control.cancelled.load(Ordering::Acquire) {
    match rx.recv_timeout(cancel_poll) {
      Ok(Ok(event)) => {
        for event in map_event(event) {
          if publish_emit(&control, event) != status::OK {
            return Ok(());
          }
        }
      }
      Ok(Err(error)) => return Err(format!("filesystem watcher failed: {error}")),
      Err(RecvTimeoutError::Timeout) => {}
      Err(RecvTimeoutError::Disconnected) => return Err("filesystem watcher channel disconnected".to_owned()),
    }
  }
  Ok(())
}

unsafe extern "C" fn cancel_watch(task_context: u64, _task_handle: u64, reason_ptr: *const u8, reason_len: usize) -> i32 {
  if reason_ptr.is_null() && reason_len != 0 {
    return status::INVALID_PAYLOAD;
  }
  let control = watch_controls()
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .get(&task_context)
    .cloned();
  let Some(control) = control else {
    return status::HANDLE_FINISHED;
  };
  control.cancelled.store(true, Ordering::Release);
  status::OK
}

unsafe fn start_fswatch_async_v1(
  request_ptr: *const u8,
  request_len: usize,
  task: *const CalcitFfiAsyncTaskV1,
  host: *const CalcitFfiAsyncHostV1,
) -> i32 {
  let task = match unsafe { copy_task_descriptor(task) } {
    Ok(task) => task,
    Err(_) => return status::INVALID_PAYLOAD,
  };
  let host = match unsafe { copy_async_host(host) } {
    Ok(host) if host.enqueue.is_some() && host.configure_task.is_some() => host,
    _ => return status::INVALID_PAYLOAD,
  };
  let args = match unsafe { decode_request(request_ptr, request_len) } {
    Ok(args) => args,
    Err(_) => return status::INVALID_PAYLOAD,
  };
  let (path, poll_interval) = match parse_options(&args) {
    Ok(options) => options,
    Err(_) => return status::INVALID_PAYLOAD,
  };

  let context = next_watch_context();
  let control = Arc::new(WatchControl {
    cancelled: AtomicBool::new(false),
    host,
    task,
  });
  watch_controls()
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .insert(context, Arc::clone(&control));
  let configure_status = configure_task(
    host,
    task,
    task_kind::STREAM,
    task_flags::SERIAL_EVENTS | task_flags::COALESCE_ALLOWED,
    context,
    cancel_watch,
  );
  if configure_status != status::OK {
    watch_controls()
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .remove(&context);
    return configure_status;
  }

  spawn(move || {
    let outcome = catch_unwind(AssertUnwindSafe(|| run_watcher(path, poll_interval, Arc::clone(&control))));
    match outcome {
      Ok(Ok(())) => {
        let _ = publish_complete(&control);
      }
      Ok(Err(error)) => {
        let _ = publish_failure(&control, error);
      }
      Err(_) => {
        let _ = publish_failure(&control, "fswatch worker panicked");
      }
    }
    watch_controls()
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .remove(&context);
  });
  status::OK
}

/// Start a cancellable filesystem event stream through async protocol v1.
///
/// # Safety
///
/// Request bytes and both descriptors must remain readable for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fswatch_calcit_ffi_async_v1(
  request_ptr: *const u8,
  request_len: usize,
  task: *const CalcitFfiAsyncTaskV1,
  host: *const CalcitFfiAsyncHostV1,
) -> i32 {
  catch_unwind(AssertUnwindSafe(|| {
    // SAFETY: forwarded from the exported C contract above.
    unsafe { start_fswatch_async_v1(request_ptr, request_len, task, host) }
  }))
  .unwrap_or(status::INTERNAL_ERROR)
}

#[cfg(test)]
mod tests {
  use super::*;
  use calcit_native_ffi::{ASYNC_PROTOCOL_VERSION, AsyncTaskCancel, encode_edn, event_kind};
  use cirru_edn::EdnListView;
  use notify::event::CreateKind;
  use std::fs;
  use std::thread::sleep;
  use std::time::{Instant, SystemTime, UNIX_EPOCH};
  use std::{ptr, slice};

  type ConfiguredTask = (u32, u32, u64, AsyncTaskCancel);
  type RecordedEvent = (u32, Vec<u8>);
  static EVENTS: OnceLock<Mutex<Vec<RecordedEvent>>> = OnceLock::new();
  static CONFIG: OnceLock<Mutex<Option<ConfiguredTask>>> = OnceLock::new();
  static QUEUE_CALLS: AtomicU64 = AtomicU64::new(0);

  unsafe extern "C" fn record_enqueue(
    _context: u64,
    _task_handle: u64,
    kind: u32,
    _response_handle: u64,
    payload_ptr: *const u8,
    payload_len: usize,
  ) -> i32 {
    let payload = if payload_len == 0 {
      vec![]
    } else {
      // SAFETY: the producer keeps payload bytes readable for this call.
      unsafe { slice::from_raw_parts(payload_ptr, payload_len) }.to_vec()
    };
    EVENTS
      .get_or_init(|| Mutex::new(vec![]))
      .lock()
      .expect("events")
      .push((kind, payload));
    status::OK
  }

  unsafe extern "C" fn record_configure(
    _context: u64,
    _task_handle: u64,
    kind: u32,
    flags: u32,
    task_context: u64,
    cancel: Option<AsyncTaskCancel>,
  ) -> i32 {
    *CONFIG.get_or_init(|| Mutex::new(None)).lock().expect("config") = cancel.map(|cancel| (kind, flags, task_context, cancel));
    status::OK
  }

  unsafe extern "C" fn always_queue_full(
    _context: u64,
    _task_handle: u64,
    _kind: u32,
    _response_handle: u64,
    _payload_ptr: *const u8,
    _payload_len: usize,
  ) -> i32 {
    QUEUE_CALLS.fetch_add(1, Ordering::Relaxed);
    status::QUEUE_FULL
  }

  fn descriptors() -> (CalcitFfiAsyncTaskV1, CalcitFfiAsyncHostV1) {
    (
      CalcitFfiAsyncTaskV1 {
        protocol_version: ASYNC_PROTOCOL_VERSION,
        struct_size: std::mem::size_of::<CalcitFfiAsyncTaskV1>() as u32,
        handle: 41,
        kind: task_kind::STREAM,
        flags: task_flags::SERIAL_EVENTS,
      },
      CalcitFfiAsyncHostV1 {
        protocol_version: ASYNC_PROTOCOL_VERSION,
        struct_size: std::mem::size_of::<CalcitFfiAsyncHostV1>() as u32,
        context: 9,
        enqueue: Some(record_enqueue),
        configure_task: Some(record_configure),
        open_response: None,
      },
    )
  }

  fn wait_for(kind: u32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
      if EVENTS
        .get_or_init(|| Mutex::new(vec![]))
        .lock()
        .expect("events")
        .iter()
        .any(|event| event.0 == kind)
      {
        return;
      }
      sleep(Duration::from_millis(5));
    }
    panic!("timed out waiting for event {kind}");
  }

  #[test]
  fn async_layout_and_event_mapping_are_stable() {
    assert_eq!(calcit_ffi_async_version(), ASYNC_PROTOCOL_VERSION);
    assert_eq!(std::mem::size_of::<CalcitFfiAsyncTaskV1>(), 24);
    assert_eq!(std::mem::size_of::<CalcitFfiAsyncHostV1>(), 40);

    let event = Event::new(EventKind::Create(CreateKind::File)).add_path(Path::new("demo.cirru").to_path_buf());
    let mapped = map_event(event);
    assert_eq!(mapped.len(), 1);
    let Edn::Struct(data) = &mapped[0] else {
      panic!("event must be a struct");
    };
    assert_eq!(
      data
        .pairs
        .iter()
        .find(|(field, _)| field.ref_str() == "type")
        .map(|(_, value)| value),
      Some(&Edn::tag("create"))
    );
  }

  #[test]
  fn emit_backpressure_remains_cancellable() {
    QUEUE_CALLS.store(0, Ordering::Relaxed);
    let (task, mut host) = descriptors();
    host.enqueue = Some(always_queue_full);
    let control = Arc::new(WatchControl {
      cancelled: AtomicBool::new(false),
      host,
      task,
    });
    let worker_control = Arc::clone(&control);
    let worker = spawn(move || publish_emit(&worker_control, Edn::tag("blocked")));
    let deadline = Instant::now() + Duration::from_secs(1);
    while QUEUE_CALLS.load(Ordering::Relaxed) == 0 && Instant::now() < deadline {
      sleep(Duration::from_millis(1));
    }
    assert!(QUEUE_CALLS.load(Ordering::Relaxed) > 0, "enqueue was not attempted");
    control.cancelled.store(true, Ordering::Release);
    assert_eq!(worker.join().expect("backpressure worker"), status::HANDLE_CLOSING);
  }

  #[test]
  fn stream_configures_coalescing_and_completes_after_cancel() {
    EVENTS.get_or_init(|| Mutex::new(vec![])).lock().expect("events").clear();
    *CONFIG.get_or_init(|| Mutex::new(None)).lock().expect("config") = None;
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
    let path = std::env::temp_dir().join(format!("calcit-fswatch-{}-{suffix}", std::process::id()));
    fs::create_dir_all(&path).expect("create test directory");
    let options = Edn::map_from_iter([
      (Edn::tag("path"), Edn::str(path.display().to_string())),
      (Edn::tag("duration"), Edn::Number(10.0)),
    ]);
    let request = encode_edn(&Edn::List(EdnListView(vec![options]))).expect("request");
    let (task, host) = descriptors();
    assert_eq!(
      unsafe { fswatch_calcit_ffi_async_v1(request.as_ptr(), request.len(), &task, &host) },
      status::OK
    );
    let (kind, flags, context, cancel) = CONFIG.get().expect("config").lock().expect("config lock").expect("configured");
    assert_eq!(kind, task_kind::STREAM);
    assert_eq!(flags, task_flags::SERIAL_EVENTS | task_flags::COALESCE_ALLOWED);
    sleep(Duration::from_millis(100));
    fs::write(path.join("event.cirru"), b"event").expect("write watched file");
    wait_for(event_kind::EMIT);
    assert_eq!(unsafe { cancel(context, task.handle, ptr::null(), 0) }, status::OK);
    wait_for(event_kind::COMPLETE);
    fs::remove_dir_all(path).expect("remove test directory");
  }

  #[test]
  fn start_rejects_invalid_payloads() {
    let (task, host) = descriptors();
    assert_eq!(
      unsafe { fswatch_calcit_ffi_async_v1(ptr::null(), 1, &task, &host) },
      status::INVALID_PAYLOAD
    );
  }
}
