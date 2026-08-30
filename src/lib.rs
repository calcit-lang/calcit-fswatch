use calcit_native_ffi::{
  BackpressurePolicy, CalcitFfiAsyncHostV1, CalcitFfiAsyncTaskV1, configure_task, copy_async_host, copy_task_descriptor,
  decode_request, encode_callback_args, enqueue_with_backpressure_until, event_kind, publish_complete as publish_async_complete,
  publish_failure as publish_async_failure, status, task_flags, task_kind,
};
use cirru_edn::{Edn, EdnStructView};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::spawn;
use std::time::{Duration, Instant};

calcit_native_ffi::export_async_abi_v1!();

struct WatchControl {
  cancelled: AtomicBool,
  host: CalcitFfiAsyncHostV1,
  task: CalcitFfiAsyncTaskV1,
}

static WATCH_CONTROLS: OnceLock<Mutex<HashMap<u64, Arc<WatchControl>>>> = OnceLock::new();
static NEXT_WATCH_CONTEXT: AtomicU64 = AtomicU64::new(1);

const WATCH_INGRESS_EVENT_CAPACITY: usize = 256;
const WATCH_INGRESS_BYTE_CAPACITY: usize = 1024 * 1024;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WatchIngressStats {
  queued_events: usize,
  queued_bytes: usize,
  high_water_events: usize,
  high_water_bytes: usize,
}

#[derive(Debug)]
struct WatchIngressState {
  queue: VecDeque<Vec<u8>>,
  queued_bytes: usize,
  #[cfg(test)]
  high_water_events: usize,
  #[cfg(test)]
  high_water_bytes: usize,
  failure: Option<String>,
}

#[derive(Debug)]
struct WatchIngress {
  event_capacity: usize,
  byte_capacity: usize,
  state: Mutex<WatchIngressState>,
  wake: Condvar,
}

enum WatchIngressReceive {
  Payload(Vec<u8>),
  Failure(String),
  Timeout,
}

impl WatchIngress {
  fn new(event_capacity: usize, byte_capacity: usize) -> Self {
    assert!(event_capacity > 0, "watch ingress event capacity must be positive");
    assert!(byte_capacity > 0, "watch ingress byte capacity must be positive");
    Self {
      event_capacity,
      byte_capacity,
      state: Mutex::new(WatchIngressState {
        queue: VecDeque::new(),
        queued_bytes: 0,
        #[cfg(test)]
        high_water_events: 0,
        #[cfg(test)]
        high_water_bytes: 0,
        failure: None,
      }),
      wake: Condvar::new(),
    }
  }

  fn fail_locked(&self, state: &mut WatchIngressState, message: String) {
    if state.failure.is_none() {
      state.queue.clear();
      state.queued_bytes = 0;
      state.failure = Some(message);
      self.wake.notify_all();
    }
  }

  fn fail(&self, message: impl Into<String>) {
    let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    self.fail_locked(&mut state, message.into());
  }

  fn push_batch(&self, payloads: Vec<Vec<u8>>) {
    if payloads.is_empty() {
      return;
    }
    let incoming_events = payloads.len();
    let Some(incoming_bytes) = payloads.iter().try_fold(0usize, |total, payload| total.checked_add(payload.len())) else {
      self.fail("fswatch ingress payload size overflow");
      return;
    };
    let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if state.failure.is_some() {
      return;
    }
    let next_events = state.queue.len().checked_add(incoming_events);
    let next_bytes = state.queued_bytes.checked_add(incoming_bytes);
    if next_events.is_none_or(|count| count > self.event_capacity) || next_bytes.is_none_or(|bytes| bytes > self.byte_capacity) {
      let message = format!(
        "fswatch ingress overflow: queued_events={} incoming_events={} event_limit={} queued_bytes={} incoming_bytes={} byte_limit={}",
        state.queue.len(),
        incoming_events,
        self.event_capacity,
        state.queued_bytes,
        incoming_bytes,
        self.byte_capacity
      );
      self.fail_locked(&mut state, message);
      return;
    }
    for payload in payloads {
      state.queued_bytes += payload.len();
      state.queue.push_back(payload);
    }
    #[cfg(test)]
    {
      state.high_water_events = state.high_water_events.max(state.queue.len());
      state.high_water_bytes = state.high_water_bytes.max(state.queued_bytes);
    }
    self.wake.notify_one();
  }

  fn failure(&self) -> Option<String> {
    self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).failure.clone()
  }

  fn receive_timeout(&self, timeout: Duration) -> WatchIngressReceive {
    let deadline = Instant::now() + timeout;
    let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    loop {
      if let Some(error) = &state.failure {
        return WatchIngressReceive::Failure(error.clone());
      }
      if let Some(payload) = state.queue.pop_front() {
        state.queued_bytes = state.queued_bytes.saturating_sub(payload.len());
        return WatchIngressReceive::Payload(payload);
      }
      let now = Instant::now();
      if now >= deadline {
        return WatchIngressReceive::Timeout;
      }
      let remaining = deadline.saturating_duration_since(now);
      let (next_state, wait) = self
        .wake
        .wait_timeout(state, remaining)
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      state = next_state;
      if wait.timed_out() && state.queue.is_empty() && state.failure.is_none() {
        return WatchIngressReceive::Timeout;
      }
    }
  }

  #[cfg(test)]
  fn stats(&self) -> WatchIngressStats {
    let state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    WatchIngressStats {
      queued_events: state.queue.len(),
      queued_bytes: state.queued_bytes,
      high_water_events: state.high_water_events,
      high_water_bytes: state.high_water_bytes,
    }
  }
}

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

fn publish_emit_payload(control: &WatchControl, ingress: &WatchIngress, payload: &[u8], policy: BackpressurePolicy) -> i32 {
  enqueue_with_backpressure_until(control.host, control.task, event_kind::EMIT, 0, payload, policy, || {
    !control.cancelled.load(Ordering::Acquire) && ingress.failure().is_none()
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

fn encode_event_batch(event: Event) -> Result<Vec<Vec<u8>>, String> {
  map_event(event)
    .into_iter()
    .map(|event| encode_callback_args(vec![event]))
    .collect()
}

fn run_ingress(
  ingress: &WatchIngress,
  cancel_poll: Duration,
  control: &WatchControl,
  policy: BackpressurePolicy,
) -> Result<(), String> {
  while !control.cancelled.load(Ordering::Acquire) {
    match ingress.receive_timeout(cancel_poll) {
      WatchIngressReceive::Payload(payload) => {
        let publish_status = publish_emit_payload(control, ingress, &payload, policy);
        if publish_status != status::OK {
          if control.cancelled.load(Ordering::Acquire) {
            return Ok(());
          }
          if let Some(error) = ingress.failure() {
            return Err(error);
          }
          return Err(format!("fswatch host rejected an ordered event with status {publish_status}"));
        }
      }
      WatchIngressReceive::Failure(error) => {
        if control.cancelled.load(Ordering::Acquire) {
          return Ok(());
        }
        return Err(error);
      }
      WatchIngressReceive::Timeout => {}
    }
  }
  Ok(())
}

fn run_watcher(path: Arc<str>, poll_interval: Duration, control: Arc<WatchControl>) -> Result<(), String> {
  let ingress = Arc::new(WatchIngress::new(WATCH_INGRESS_EVENT_CAPACITY, WATCH_INGRESS_BYTE_CAPACITY));
  let callback_ingress = Arc::clone(&ingress);
  let config = notify::Config::default().with_poll_interval(poll_interval);
  let mut watcher = RecommendedWatcher::new(
    move |result| match result {
      Ok(event) => match encode_event_batch(event) {
        Ok(payloads) => callback_ingress.push_batch(payloads),
        Err(error) => callback_ingress.fail(error),
      },
      Err(error) => callback_ingress.fail(format!("filesystem watcher failed: {error}")),
    },
    config,
  )
  .map_err(|error| format!("failed to create watcher: {error}"))?;
  watcher
    .watch(Path::new(&*path), RecursiveMode::Recursive)
    .map_err(|error| format!("failed to watch path {path}: {error}"))?;

  let cancel_poll = poll_interval.min(Duration::from_millis(100));
  run_ingress(&ingress, cancel_poll, &control, BackpressurePolicy::default())
}

fn publish_terminal(control: &WatchControl, outcome: Result<(), String>) -> i32 {
  match outcome {
    Ok(()) => publish_complete(control),
    Err(error) => publish_failure(control, error),
  }
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
  let configure_status = configure_task(host, task, task_kind::STREAM, task_flags::SERIAL_EVENTS, context, cancel_watch);
  if configure_status != status::OK {
    watch_controls()
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .remove(&context);
    return configure_status;
  }

  spawn(move || {
    let outcome = match catch_unwind(AssertUnwindSafe(|| run_watcher(path, poll_interval, Arc::clone(&control)))) {
      Ok(outcome) => outcome,
      Err(_) => Err("fswatch worker panicked".to_owned()),
    };
    let _ = publish_terminal(&control, outcome);
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
  use notify::event::{CreateKind, RemoveKind, RenameMode};
  use std::fs;
  use std::thread::sleep;
  use std::time::{Instant, SystemTime, UNIX_EPOCH};
  use std::{ptr, slice};

  type ConfiguredTask = (u32, u32, u64, AsyncTaskCancel);
  type RecordedEvent = (u32, Vec<u8>);
  static EVENTS: OnceLock<Mutex<Vec<RecordedEvent>>> = OnceLock::new();
  static CONFIG: OnceLock<Mutex<Option<ConfiguredTask>>> = OnceLock::new();
  static TEST_LOCK: Mutex<()> = Mutex::new(());
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

  fn event_seen(kind: u32) -> bool {
    EVENTS
      .get_or_init(|| Mutex::new(vec![]))
      .lock()
      .expect("events")
      .iter()
      .any(|event| event.0 == kind)
  }

  fn event_count(kind: u32) -> usize {
    EVENTS
      .get_or_init(|| Mutex::new(vec![]))
      .lock()
      .expect("events")
      .iter()
      .filter(|event| event.0 == kind)
      .count()
  }

  fn payload_seen(fragment: &str) -> bool {
    EVENTS
      .get_or_init(|| Mutex::new(vec![]))
      .lock()
      .expect("events")
      .iter()
      .filter(|event| event.0 == event_kind::EMIT)
      .any(|event| String::from_utf8_lossy(&event.1).contains(fragment))
  }

  fn wait_for_payload(fragment: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
      if payload_seen(fragment) {
        return;
      }
      sleep(Duration::from_millis(5));
    }
    let events = EVENTS.get_or_init(|| Mutex::new(vec![])).lock().expect("events").clone();
    panic!("timed out waiting for payload {fragment:?}; recorded events: {events:?}");
  }

  fn wait_for(kind: u32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
      if event_seen(kind) {
        return;
      }
      sleep(Duration::from_millis(5));
    }
    let events = EVENTS.get_or_init(|| Mutex::new(vec![])).lock().expect("events").clone();
    panic!("timed out waiting for event {kind}; recorded events: {events:?}");
  }

  #[test]
  fn async_layout_and_event_mapping_are_stable() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
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

    let old_path = Path::new("old.cirru").to_path_buf();
    let new_path = Path::new("new.cirru").to_path_buf();
    let rename = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
      .add_path(old_path.clone())
      .add_path(new_path.clone());
    let mapped = map_event(rename);
    assert_eq!(mapped.len(), 2);
    let paths = mapped
      .iter()
      .map(|event| {
        let Edn::Struct(data) = event else {
          panic!("event must be a struct");
        };
        data
          .pairs
          .iter()
          .find(|(field, _)| field.ref_str() == "path")
          .map(|(_, value)| value)
          .expect("event path")
      })
      .collect::<Vec<_>>();
    assert_eq!(
      paths,
      vec![&Edn::str(old_path.display().to_string()), &Edn::str(new_path.display().to_string())]
    );

    for (event, expected_type) in [
      (
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content))).add_path(Path::new("modify.cirru").to_path_buf()),
        "modify",
      ),
      (
        Event::new(EventKind::Remove(RemoveKind::File)).add_path(Path::new("remove.cirru").to_path_buf()),
        "remove",
      ),
    ] {
      let mapped = map_event(event);
      let Edn::Struct(data) = &mapped[0] else {
        panic!("event must be a struct");
      };
      assert_eq!(
        data
          .pairs
          .iter()
          .find(|(field, _)| field.ref_str() == "type")
          .map(|(_, value)| value),
        Some(&Edn::tag(expected_type))
      );
    }
  }

  #[test]
  fn emit_backpressure_remains_cancellable() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    QUEUE_CALLS.store(0, Ordering::Relaxed);
    let (task, mut host) = descriptors();
    host.enqueue = Some(always_queue_full);
    let control = Arc::new(WatchControl {
      cancelled: AtomicBool::new(false),
      host,
      task,
    });
    let ingress = Arc::new(WatchIngress::new(1, 1024));
    let payload = encode_callback_args(vec![Edn::tag("blocked")]).expect("payload");
    let worker_control = Arc::clone(&control);
    let worker_ingress = Arc::clone(&ingress);
    let worker = spawn(move || publish_emit_payload(&worker_control, &worker_ingress, &payload, BackpressurePolicy::default()));
    let deadline = Instant::now() + Duration::from_secs(1);
    while QUEUE_CALLS.load(Ordering::Relaxed) == 0 && Instant::now() < deadline {
      sleep(Duration::from_millis(1));
    }
    assert!(QUEUE_CALLS.load(Ordering::Relaxed) > 0, "enqueue was not attempted");
    control.cancelled.store(true, Ordering::Release);
    assert_eq!(worker.join().expect("backpressure worker"), status::HANDLE_CLOSING);
  }

  #[test]
  fn ingress_overflow_is_bounded_and_terminal() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    EVENTS.get_or_init(|| Mutex::new(vec![])).lock().expect("events").clear();
    let (task, host) = descriptors();
    let control = WatchControl {
      cancelled: AtomicBool::new(false),
      host,
      task,
    };
    let ingress = WatchIngress::new(8, 256);
    for index in 0..10_000 {
      ingress.push_batch(vec![format!("event-{index}").into_bytes()]);
    }
    let failure = ingress.failure().expect("overflow failure");
    assert!(failure.contains("fswatch ingress overflow"));
    let stats = ingress.stats();
    assert_eq!(stats.queued_events, 0);
    assert_eq!(stats.queued_bytes, 0);
    assert!(stats.high_water_events <= 8);
    assert!(stats.high_water_bytes <= 256);
    assert!(run_ingress(&ingress, Duration::from_millis(1), &control, BackpressurePolicy::default()).is_err());
    assert_eq!(publish_terminal(&control, Err(failure)), status::OK);
    assert_eq!(event_count(event_kind::FAIL), 1);
    assert_eq!(event_count(event_kind::COMPLETE), 0);
  }

  #[test]
  fn ingress_preserves_order_and_rejects_batches_atomically() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let ingress = WatchIngress::new(3, 64);
    ingress.push_batch(vec![b"first".to_vec(), b"second".to_vec()]);
    assert!(matches!(
      ingress.receive_timeout(Duration::ZERO),
      WatchIngressReceive::Payload(payload) if payload == b"first"
    ));
    assert!(matches!(
      ingress.receive_timeout(Duration::ZERO),
      WatchIngressReceive::Payload(payload) if payload == b"second"
    ));

    ingress.push_batch(vec![b"kept".to_vec()]);
    ingress.push_batch(vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]);
    assert!(matches!(ingress.receive_timeout(Duration::ZERO), WatchIngressReceive::Failure(_)));
    let stats = ingress.stats();
    assert_eq!(stats.queued_events, 0);
    assert_eq!(stats.queued_bytes, 0);
  }

  #[test]
  fn cancellation_completes_once_without_failure() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    EVENTS.get_or_init(|| Mutex::new(vec![])).lock().expect("events").clear();
    let (task, host) = descriptors();
    let control = WatchControl {
      cancelled: AtomicBool::new(true),
      host,
      task,
    };
    let ingress = WatchIngress::new(1, 64);
    let outcome = run_ingress(&ingress, Duration::from_millis(1), &control, BackpressurePolicy::default());
    assert_eq!(outcome, Ok(()));
    assert_eq!(publish_terminal(&control, outcome), status::OK);
    assert_eq!(event_count(event_kind::COMPLETE), 1);
    assert_eq!(event_count(event_kind::FAIL), 0);
  }

  #[test]
  #[cfg_attr(
    target_os = "macos",
    ignore = "FSEvents does not reliably report changes from temporary Git worktrees"
  )]
  fn stream_preserves_real_event_kinds_and_completes_once_after_cancel() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    EVENTS.get_or_init(|| Mutex::new(vec![])).lock().expect("events").clear();
    *CONFIG.get_or_init(|| Mutex::new(None)).lock().expect("config") = None;
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
    let path = std::env::current_dir()
      .expect("current directory")
      .join(format!(".calcit-fswatch-test-{}-{suffix}", std::process::id()));
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
    assert_eq!(flags, task_flags::SERIAL_EVENTS);
    // The async start contract reports watcher initialization failures through
    // a terminal event, so configuration may precede OS watcher readiness.
    // Keep producing distinct create events until the stream proves ready
    // instead of relying on an arbitrary startup sleep.
    let ready_deadline = Instant::now() + Duration::from_secs(5);
    let mut sequence = 0;
    while !event_seen(event_kind::EMIT) && Instant::now() < ready_deadline {
      fs::write(path.join(format!("event-{sequence}.cirru")), b"event").expect("write watched file");
      sequence += 1;
      sleep(Duration::from_millis(25));
    }
    wait_for(event_kind::EMIT);
    wait_for_payload(":create");

    let source = path.join("lifecycle.cirru");
    let renamed = path.join("renamed.cirru");
    fs::write(&source, b"first").expect("create lifecycle file");
    fs::write(&source, b"second").expect("modify lifecycle file");
    fs::rename(&source, &renamed).expect("rename lifecycle file");
    fs::remove_file(&renamed).expect("remove lifecycle file");
    wait_for_payload(":modify");
    wait_for_payload(":rename");
    wait_for_payload(":remove");

    assert_eq!(unsafe { cancel(context, task.handle, ptr::null(), 0) }, status::OK);
    wait_for(event_kind::COMPLETE);
    sleep(Duration::from_millis(50));
    assert_eq!(event_count(event_kind::COMPLETE), 1);
    assert_eq!(event_count(event_kind::FAIL), 0);
    fs::remove_dir_all(path).expect("remove test directory");
  }

  #[test]
  fn start_rejects_invalid_payloads() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let (task, host) = descriptors();
    assert_eq!(
      unsafe { fswatch_calcit_ffi_async_v1(ptr::null(), 1, &task, &host) },
      status::INVALID_PAYLOAD
    );
  }
}
