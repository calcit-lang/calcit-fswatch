use cirru_edn::{Edn, EdnListView, EdnStructView};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{sleep, spawn};
use std::time::Duration;

const PROTOCOL_VERSION: u32 = 1;
const STATUS_OK: i32 = 0;
const STATUS_HANDLE_FINISHED: i32 = 4;
const STATUS_QUEUE_FULL: i32 = 7;
const STATUS_INVALID_PAYLOAD: i32 = 8;
const STATUS_INTERNAL_ERROR: i32 = 9;
const TASK_STREAM: u32 = 2;
const TASK_SERIAL_EVENTS: u32 = 1;
const TASK_COALESCE_ALLOWED: u32 = 1 << 1;
const EVENT_EMIT: u32 = 1;
const EVENT_COMPLETE: u32 = 2;
const EVENT_FAIL: u32 = 3;
const MAX_BUFFER_BYTES: usize = 256 * 1024 * 1024;

type AsyncHostEnqueue = unsafe extern "C" fn(u64, u64, u32, u64, *const u8, usize) -> i32;
type AsyncTaskCancel = unsafe extern "C" fn(u64, u64, *const u8, usize) -> i32;
type AsyncResponseResolve = unsafe extern "C" fn(u64, u64, u32, *const u8, usize) -> i32;
type AsyncHostConfigure = unsafe extern "C" fn(u64, u64, u32, u32, u64, Option<AsyncTaskCancel>) -> i32;
type AsyncHostOpenResponse = unsafe extern "C" fn(u64, u64, u64, u64, Option<AsyncResponseResolve>, *mut u64) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CalcitFfiAsyncTaskV1 {
  protocol_version: u32,
  struct_size: u32,
  handle: u64,
  kind: u32,
  flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CalcitFfiAsyncHostV1 {
  protocol_version: u32,
  struct_size: u32,
  context: u64,
  enqueue: Option<AsyncHostEnqueue>,
  configure_task: Option<AsyncHostConfigure>,
  open_response: Option<AsyncHostOpenResponse>,
}

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

unsafe fn read_abi_header<T>(value: *const T) -> Result<(u32, u32), i32> {
  if value.is_null() {
    return Err(STATUS_INVALID_PAYLOAD);
  }
  let bytes = value.cast::<u8>();
  // SAFETY: every versioned descriptor begins with two readable u32 fields.
  let protocol_version = unsafe { ptr::read_unaligned(bytes.cast::<u32>()) };
  // SAFETY: the second header field begins four bytes after the first.
  let struct_size = unsafe { ptr::read_unaligned(bytes.add(std::mem::size_of::<u32>()).cast::<u32>()) };
  Ok((protocol_version, struct_size))
}

unsafe fn copy_task(value: *const CalcitFfiAsyncTaskV1) -> Result<CalcitFfiAsyncTaskV1, i32> {
  // SAFETY: forwarded from the versioned descriptor contract.
  let (version, size) = unsafe { read_abi_header(value) }?;
  if version != PROTOCOL_VERSION || size < std::mem::size_of::<CalcitFfiAsyncTaskV1>() as u32 {
    return Err(STATUS_INVALID_PAYLOAD);
  }
  // SAFETY: the validated size covers every v1 field.
  Ok(unsafe { ptr::read_unaligned(value) })
}

unsafe fn copy_host(value: *const CalcitFfiAsyncHostV1) -> Result<CalcitFfiAsyncHostV1, i32> {
  // SAFETY: forwarded from the versioned descriptor contract.
  let (version, size) = unsafe { read_abi_header(value) }?;
  if version != PROTOCOL_VERSION || size < std::mem::size_of::<CalcitFfiAsyncHostV1>() as u32 {
    return Err(STATUS_INVALID_PAYLOAD);
  }
  // SAFETY: the validated size covers every v1 field.
  Ok(unsafe { ptr::read_unaligned(value) })
}

unsafe fn decode_request(request_ptr: *const u8, request_len: usize) -> Result<Vec<Edn>, String> {
  if request_len > MAX_BUFFER_BYTES {
    return Err(format!("fswatch request exceeds {MAX_BUFFER_BYTES} bytes"));
  }
  if request_ptr.is_null() && request_len != 0 {
    return Err("fswatch request pointer is null".to_owned());
  }
  let request = if request_len == 0 {
    &[]
  } else {
    // SAFETY: the host keeps request bytes readable for this start call.
    unsafe { slice::from_raw_parts(request_ptr, request_len) }
  };
  let source = std::str::from_utf8(request).map_err(|error| format!("fswatch request is not UTF-8: {error}"))?;
  let data = cirru_edn::parse(source).map_err(|error| format!("fswatch request is not valid Cirru EDN: {error}"))?;
  let Edn::List(EdnListView(args)) = data else {
    return Err("fswatch request must be a Cirru EDN list".to_owned());
  };
  Ok(args)
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

fn encode_edn(value: &Edn) -> Result<Vec<u8>, String> {
  cirru_edn::format(value, true)
    .map(String::into_bytes)
    .map_err(|error| format!("failed to encode fswatch payload: {error}"))
}

fn enqueue(control: &WatchControl, kind: u32, payload: &[u8], stop_when_cancelled: bool) -> i32 {
  let Some(enqueue) = control.host.enqueue else {
    return STATUS_INVALID_PAYLOAD;
  };
  loop {
    if stop_when_cancelled && control.cancelled.load(Ordering::Acquire) {
      return STATUS_HANDLE_FINISHED;
    }
    // SAFETY: copied descriptors remain valid while the host is running and payload is readable for this call.
    let status = unsafe { enqueue(control.host.context, control.task.handle, kind, 0, payload.as_ptr(), payload.len()) };
    if status != STATUS_QUEUE_FULL {
      return status;
    }
    sleep(Duration::from_millis(1));
  }
}

fn publish_emit(control: &WatchControl, event: Edn) -> i32 {
  let payload = match encode_edn(&Edn::List(EdnListView(vec![event]))) {
    Ok(payload) => payload,
    Err(_) => return STATUS_INTERNAL_ERROR,
  };
  enqueue(control, EVENT_EMIT, &payload, true)
}

fn publish_failure(control: &WatchControl, message: impl Into<String>) -> i32 {
  let payload = encode_edn(&Edn::str(message.into())).unwrap_or_else(|_| b"|failed-to-encode-fswatch-error".to_vec());
  enqueue(control, EVENT_FAIL, &payload, false)
}

fn publish_complete(control: &WatchControl) -> i32 {
  enqueue(control, EVENT_COMPLETE, b"&unit", false)
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
          if publish_emit(&control, event) != STATUS_OK {
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
    return STATUS_INVALID_PAYLOAD;
  }
  let control = watch_controls()
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .get(&task_context)
    .cloned();
  let Some(control) = control else {
    return STATUS_HANDLE_FINISHED;
  };
  control.cancelled.store(true, Ordering::Release);
  STATUS_OK
}

unsafe fn start_fswatch_async_v1(
  request_ptr: *const u8,
  request_len: usize,
  task: *const CalcitFfiAsyncTaskV1,
  host: *const CalcitFfiAsyncHostV1,
) -> i32 {
  let task = match unsafe { copy_task(task) } {
    Ok(task) => task,
    Err(status) => return status,
  };
  let host = match unsafe { copy_host(host) } {
    Ok(host) if host.enqueue.is_some() && host.configure_task.is_some() => host,
    _ => return STATUS_INVALID_PAYLOAD,
  };
  let args = match unsafe { decode_request(request_ptr, request_len) } {
    Ok(args) => args,
    Err(_) => return STATUS_INVALID_PAYLOAD,
  };
  let (path, poll_interval) = match parse_options(&args) {
    Ok(options) => options,
    Err(_) => return STATUS_INVALID_PAYLOAD,
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
  let Some(configure) = host.configure_task else {
    return STATUS_INVALID_PAYLOAD;
  };
  // SAFETY: copied host function pointers remain valid while the host runs.
  let status = unsafe {
    configure(
      host.context,
      task.handle,
      TASK_STREAM,
      TASK_SERIAL_EVENTS | TASK_COALESCE_ALLOWED,
      context,
      Some(cancel_watch),
    )
  };
  if status != STATUS_OK {
    watch_controls()
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .remove(&context);
    return status;
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
  STATUS_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn calcit_ffi_async_version() -> u32 {
  PROTOCOL_VERSION
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
  .unwrap_or(STATUS_INTERNAL_ERROR)
}

#[cfg(test)]
mod tests {
  use super::*;
  use notify::event::CreateKind;
  use std::fs;
  use std::time::{Instant, SystemTime, UNIX_EPOCH};

  type ConfiguredTask = (u32, u32, u64, AsyncTaskCancel);
  type RecordedEvent = (u32, Vec<u8>);
  static EVENTS: OnceLock<Mutex<Vec<RecordedEvent>>> = OnceLock::new();
  static CONFIG: OnceLock<Mutex<Option<ConfiguredTask>>> = OnceLock::new();

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
    STATUS_OK
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
    STATUS_OK
  }

  fn descriptors() -> (CalcitFfiAsyncTaskV1, CalcitFfiAsyncHostV1) {
    (
      CalcitFfiAsyncTaskV1 {
        protocol_version: PROTOCOL_VERSION,
        struct_size: std::mem::size_of::<CalcitFfiAsyncTaskV1>() as u32,
        handle: 41,
        kind: TASK_STREAM,
        flags: TASK_SERIAL_EVENTS,
      },
      CalcitFfiAsyncHostV1 {
        protocol_version: PROTOCOL_VERSION,
        struct_size: std::mem::size_of::<CalcitFfiAsyncHostV1>() as u32,
        context: 9,
        enqueue: Some(record_enqueue),
        configure_task: Some(record_configure),
        open_response: None,
      },
    )
  }

  fn wait_for(kind: u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
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
    assert_eq!(calcit_ffi_async_version(), 1);
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
      STATUS_OK
    );
    let (kind, flags, context, cancel) = CONFIG.get().expect("config").lock().expect("config lock").expect("configured");
    assert_eq!(kind, TASK_STREAM);
    assert_eq!(flags, TASK_SERIAL_EVENTS | TASK_COALESCE_ALLOWED);
    assert_eq!(unsafe { cancel(context, task.handle, ptr::null(), 0) }, STATUS_OK);
    wait_for(EVENT_COMPLETE);
    fs::remove_dir_all(path).expect("remove test directory");
  }

  #[test]
  fn start_rejects_invalid_payloads() {
    let (task, host) = descriptors();
    assert_eq!(
      unsafe { fswatch_calcit_ffi_async_v1(ptr::null(), 1, &task, &host) },
      STATUS_INVALID_PAYLOAD
    );
  }
}
