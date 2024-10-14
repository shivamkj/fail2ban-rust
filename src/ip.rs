use std::collections::HashMap;
use std::time::{Duration, Instant};

// Stores IPs and the timestamps of their occurrences
pub struct Tracker {
  occurrences: HashMap<String, Vec<Instant>>, // IP -> list of occurrence timestamps
}

impl Tracker {
  pub fn new() -> Self {
    Self {
      occurrences: HashMap::new(),
    }
  }

  pub fn add_ip(&mut self, ip: &str, findtime: Duration) {
    let now = Instant::now();
    let entry = self.occurrences.entry(ip.to_string()).or_insert_with(Vec::new);

    // Add the new timestamp
    entry.push(now);

    // Remove occurrences older than findtime
    entry.retain(|&timestamp| now.duration_since(timestamp) < findtime);
  }

  pub fn check_ip(&self, ip: &str, maxretry: usize) -> bool {
    if let Some(occurrences) = self.occurrences.get(ip) {
      return occurrences.len() >= maxretry;
    }
    false
  }
}
