use crate::expr;
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::time::{self, Duration};

// Stores IPs and the timestamps of their occurrences
pub struct Tracker {
  occurrences: Arc<Mutex<HashMap<String, usize>>>, // { IP : count of violations}
}

impl Tracker {
  pub fn new() -> Self {
    Self {
      occurrences: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn add_ip(&self, ip: String, findtime: Duration) {
    let map_clone = Arc::clone(&self.occurrences); // Clone the Arc to move into the async task
    {
      // Increment the occurrence of IP by 1
      let mut map = self.occurrences.lock().unwrap();
      let current_count = *map.get(&ip).unwrap_or(&0) + 1;
      map.insert(ip.to_string(), current_count);
    }

    // Spawn a task to remove the key after findtime
    task::spawn(async move {
      time::sleep(findtime).await;
      let mut map = map_clone.lock().unwrap();
      map.remove(&ip); // Safely remove the key after TTL expires
    });
  }

  pub fn check_ip(&self, ip: &str, maxretry: usize) -> bool {
    let map = self.occurrences.lock().unwrap(); // Lock the map for reading
    if let Some(occurrences) = map.get(ip) {
      return *occurrences >= maxretry;
    }
    false
  }

  pub fn block_ip(&self, ip: String) -> () {
    // Basic IP validation (you might want to add more thorough validation)
    if !ip.split('.').all(|octet| octet.parse::<u8>().is_ok()) {
      return eprintln!("Invalid IP Address: {}", ip);
    }

    if is_ip_blocked(&ip) {
      return;
    }

    // Using iptables on Linux
    let output = Command::new("sudo")
      .args(["iptables", "-A", "INPUT", "-s", &ip, "-j", "DROP"])
      .output()
      .map_err(|e| eprintln!("Error while running command (IPv4): {}", e.to_string()));
    let output = expr!(output, return);

    if !output.status.success() {
      let error = String::from_utf8_lossy(&output.stderr);
      return eprintln!("Error occured while blocking IPv4: {}", error.to_string());
    }
    println!("IP {} has been blocked", ip);

    // Spawn a task to remove the key after findtime
    task::spawn(async move {
      time::sleep(Duration::from_secs(60)).await;
      let _ = unblock_ip(&ip);
    });
  }
}

fn is_ip_blocked(ip: &str) -> bool {
  let output = Command::new("sudo")
    .args(["iptables", "-C", "INPUT", "-s", ip, "-j", "DROP"])
    .output();

  // If command succeeds (returns 0), the rule exists
  matches!(output, Ok(output) if output.status.success())
}

pub fn unblock_ip(ip: &str) -> Result<(), Box<dyn Error>> {
  if !is_ip_blocked(ip) {
    return Ok(());
  }

  // Unblock IP
  let output = Command::new("sudo")
    .args(["iptables", "-D", "INPUT", "-s", ip, "-j", "DROP"])
    .output()?;

  if output.status.success() {
    println!("IP {} has been unblocked", ip);
  } else {
    let error = String::from_utf8_lossy(&output.stderr);
    eprintln!("Failed to unblock IP: {}", error);
  }

  Ok(())
}
