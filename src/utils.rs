use regex::Regex;
use serde::Deserialize;
use std::fs::{read_dir, read_to_string, File};
use std::path::Path;
use std::time::Duration;

const IP_REGEX: &str = r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})";

#[derive(Deserialize)]
struct Config {
  watcher: Vec<WatchConfigRaw>,
}

#[derive(Deserialize)]
struct WatchConfigRaw {
  path: String,
  regex: String,   // Regular expression for matching IP addresses or other patterns
  findtime: u64,   // Duration (in seconds) to track occurrences
  maxretry: usize, // Threshold for number of occurrences
}

pub struct WatchConfig {
  pub path: String,
  pub regex: Regex,       // Pre-compiled regex
  pub findtime: Duration, // Duration to track IP occurrences
  pub maxretry: usize,    // Threshold for number of occurrences
}

// Function to read all TOML config files in config directory and load them
pub fn load_config(dir: &str) -> Vec<WatchConfig> {
  let mut all_configs: Vec<WatchConfig> = Vec::new();

  // Read all files in the directory
  for entry in read_dir(dir).expect("Failed to read config directory") {
    let entry = entry.expect("Failed to read file in config directory");
    let path = entry.path();

    // Check if the file has a .toml extension
    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
      println!("Loading config from: {:?}", path);

      // Load and parse the config file
      let content = read_to_string(&path).expect("Failed to read config file");
      let parsed_config: Config = toml::from_str(&content).expect("Failed to parse config");

      // Pre-compile the regex for each watch configuration
      for config in parsed_config.watcher {
        let regex = &config.regex.replace("<IP>", IP_REGEX);
        let regex = Regex::new(regex).expect("Invalid regex");

        all_configs.push(WatchConfig {
          path: config.path,
          regex,
          findtime: Duration::from_secs(config.findtime),
          maxretry: config.maxretry,
        });
      }
    }
  }

  all_configs
}

// Function to get the file size
pub fn file_length(path: &str) -> u64 {
  if let Ok(file) = File::open(Path::new(path)) {
    file.metadata().expect("Error while fetching file metadata").len()
  } else {
    0
  }
}
