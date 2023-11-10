use std::collections::HashMap;
use std::sync::Arc;

use cirru_edn::Edn;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

#[no_mangle]
pub fn abi_version() -> String {
  String::from("0.0.6")
}

#[no_mangle]
pub fn fswatch(
  args: Vec<Edn>,
  handler: Arc<dyn Fn(Vec<Edn>) -> Result<Edn, String> + Send + Sync + 'static>,
  _finish: Box<dyn FnOnce()>,
) -> Result<Edn, String> {
  if let Some(options) = args.get(0) {
    match options {
      Edn::Map(o) => {
        let path = &*o.get(&Edn::tag("path")).ok_or("path is required")?.read_str()?;
        let duration = o.get(&Edn::tag("duration")).ok_or("duration is required")?.read_number()? as u64;

        // Create a channel to receive the events.
        let (tx, rx) = channel();

        println!("watching path: {} every {}ms", path, duration);

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_millis(duration)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(path, RecursiveMode::Recursive).unwrap();

        loop {
          match rx.recv() {
            Ok(event) => match event {
              DebouncedEvent::Write(path) => {
                handler(vec![Edn::Map(HashMap::from([
                  (Edn::tag("type"), Edn::tag("wrote")),
                  (Edn::tag("path"), Edn::str(&path.display().to_string())),
                ]))])?;
              }
              DebouncedEvent::Create(path) => {
                handler(vec![Edn::Map(HashMap::from([
                  (Edn::tag("type"), Edn::tag("created")),
                  (Edn::tag("path"), Edn::str(&path.display().to_string())),
                ]))])?;
              }
              DebouncedEvent::Remove(path) => {
                handler(vec![Edn::Map(HashMap::from([
                  (Edn::tag("type"), Edn::tag("removed")),
                  (Edn::tag("path"), Edn::str(&path.display().to_string())),
                ]))])?;
              }
              DebouncedEvent::Rename(from, to) => {
                handler(vec![Edn::Map(HashMap::from([
                  (Edn::tag("type"), Edn::tag("renamed")),
                  (Edn::tag("path"), Edn::str(&to.display().to_string())),
                  (Edn::tag("from"), Edn::str(&from.display().to_string())),
                ]))])?;
              }
              DebouncedEvent::NoticeWrite(_) | DebouncedEvent::NoticeRemove(_) => {}
              _ => println!("skipped event: {:?}", event),
            },
            Err(e) => println!("watch error: {:?}", e),
          }
        }
      }
      _ => Err(format!("invalid options: {:?}", options)),
    }
  } else {
    Err(String::from("missing options"))
  }
}
