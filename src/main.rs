use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc::channel;
use utils::WatchConfig;
mod ip;
mod utils;

// Function to search for IP addresses or other patterns using regex
fn search_ip_in_new_lines(ip_tracker: &mut ip::Tracker, start_line: u64, config: &WatchConfig) {
  if let Ok(mut file) = File::open(&config.path) {
    let _ = file.seek(SeekFrom::Start(start_line));
    let reader = BufReader::new(file);

    for (_, line) in reader.lines().enumerate() {
      if let Ok(line_content) = line {
        if let Some(ip_match) = config.regex.captures(&line_content).and_then(|v| v.get(1)) {
          let ip_str = &ip_match.as_str();
          // Add the IP occurrence
          ip_tracker.add_ip(ip_str, config.findtime);

          // Check if the IP exceeded the threshold
          if ip_tracker.check_ip(ip_str, config.maxretry) {
            println!(
              "IP {} found more than {} times in the last {:?} seconds",
              ip_str,
              config.maxretry,
              config.findtime.as_secs()
            );
          };
        }
      }
    }
  }
}

fn watch_files(all_config: Vec<utils::WatchConfig>) -> notify::Result<()> {
  let (tx, rx) = channel();

  // Automatically select the best implementation for your platform.
  // You can also access each implementation directly e.g. INotifyWatcher.
  let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

  // HashMap to track total bytes in each file
  let mut file_bytes = HashMap::new();

  // HashMap to track IP occurrences for each file
  let mut ip_trackers = HashMap::new();

  // Add a path to be watched. All files and directories at that path and
  // below will be monitored for changes.
  for config in &all_config {
    watcher.watch(Path::new(&config.path), RecursiveMode::NonRecursive)?;
    file_bytes.insert(config.path.clone(), utils::file_length(&config.path));
    ip_trackers.insert(config.path.clone(), ip::Tracker::new());
  }

  println!("Watching files...");

  for res in rx {
    match res {
      Ok(event) => {
        let path_str = event.paths[0].to_str().expect("msg");
        let previous_len = file_bytes.get(path_str).cloned().unwrap_or(0);
        // Get the current file length
        let current_len = utils::file_length(path_str);

        println!("File changed: {} {:?}", path_str, event);

        // If new lines were added
        if current_len > previous_len {
          // Find the corresponding config for the file
          if let Some(config) = &all_config.iter().find(|c| c.path == path_str) {
            // Search for IP addresses or pattern in new lines
            if let Some(ip_tracker) = ip_trackers.get_mut(path_str) {
              search_ip_in_new_lines(ip_tracker, previous_len, &config);
            }
          }
        }

        // Update the line count in the HashMap
        file_bytes.insert(path_str.to_string(), current_len);
      }
      Err(error) => println!("Error Occured while watching file: {error:?}"),
    }
  }

  Ok(())
}

fn main() {
  // Load all configs from the "configs" directory
  let configs = utils::load_config("./test-config");

  // Watch files
  if let Err(error) = watch_files(configs) {
    println!("Error Occured while watching: {error:?}");
  }
}
