use std::sync::Arc;
use std::{collections::HashMap, path::Path};

use cirru_edn::Edn;
use notify::event::{DataChange, ModifyKind};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;

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
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(Path::new(path), RecursiveMode::Recursive).unwrap();

        for res in rx {
          match res {
            Ok(event) => match event.kind {
              EventKind::Modify(m) => match m {
                ModifyKind::Data(change) => match change {
                  DataChange::Content => {
                    for path in event.paths {
                      handler(vec![new_event("modify", &path.display().to_string(), &format!("{:?}", m))])?;
                    }
                  }
                  DataChange::Size => {}
                  DataChange::Any => {}
                  DataChange::Other => {}
                },
                ModifyKind::Name(_) => {
                  for path in event.paths {
                    handler(vec![new_event("rename", &path.display().to_string(), &format!("{:?}", m))])?;
                  }
                }
                ModifyKind::Any => {}
                ModifyKind::Metadata(_m) => {}
                ModifyKind::Other => {}
              },
              EventKind::Create(m) => {
                for path in event.paths {
                  handler(vec![new_event("create", &path.display().to_string(), &format!("{:?}", m))])?;
                }
              }
              EventKind::Remove(m) => {
                for path in event.paths {
                  handler(vec![new_event("remove", &path.display().to_string(), &format!("{:?}", m))])?;
                }
              }
              _ => {}
            },
            Err(error) => println!("error: {error}"),
          }
        }
        Ok(Edn::Nil)
      }
      _ => Err(format!("invalid options: {:?}", options)),
    }
  } else {
    Err(String::from("missing options"))
  }
}

fn new_event(t: &str, p: &str, extra: &str) -> Edn {
  Edn::Map(HashMap::from([
    (Edn::tag("type"), Edn::tag(t)),
    (Edn::tag("path"), Edn::str(p)),
    (Edn::tag("extra"), Edn::str(extra)),
  ]))
}
