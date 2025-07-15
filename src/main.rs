//! Solana Leader Schedule CLI Tool
//!
//! A command-line tool to fetch and display Solana validator leader schedule information.
//! Shows when a validator is scheduled to be a leader in a specific epoch with accurate timestamps.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tabled::{Table, Tabled};

/// Approximate slots per second for Solana network
const SLOTS_PER_SECOND: f64 = 2.5;

/// Command-line arguments structure
#[derive(Parser)]
#[command(name = "leader_schedule")]
#[command(about = "Get validator leader schedule information")]
#[command(version)]
struct Args {
    /// Validator identity public key
    #[arg(short, long)]
    identity: String,

    /// Solana cluster (mainnet, testnet, devnet)
    #[arg(short, long, default_value = "mainnet")]
    cluster: String,

    /// Epoch number (if not specified, uses current epoch)
    #[arg(short, long)]
    epoch: Option<u64>,

    /// Show all slots (including past ones). Default: show only upcoming slots
    #[arg(long)]
    all: bool,
}

/// Represents a leader slot with its timing information
#[derive(Tabled)]
struct LeaderSlot {
    #[tabled(rename = "Epoch")]
    epoch: u64,
    #[tabled(rename = "Slot")]
    slot: u64,
    #[tabled(rename = "Time (Local)")]
    time_local: String,
    #[tabled(rename = "Time Diff")]
    time_diff: String,
}

/// Returns the RPC URL for the specified cluster
fn get_cluster_url(cluster: &str) -> Result<String> {
    match cluster.to_lowercase().as_str() {
        "mainnet" | "mainnet-beta" => Ok("https://api.mainnet-beta.solana.com".to_string()),
        "testnet" => Ok("https://api.testnet.solana.com".to_string()),
        "devnet" => Ok("https://api.devnet.solana.com".to_string()),
        _ => Err(anyhow!(
            "Invalid cluster: {}. Supported clusters: mainnet, testnet, devnet",
            cluster
        )),
    }
}

/// Converts a slot number to a local timestamp using current network time as reference
///
/// Uses the current slot and timestamp from the network to calculate accurate timestamps
/// based on the slot-to-time relationship.
fn slot_to_timestamp_local(
    slot: u64,
    slots_per_second: f64,
    current_slot: u64,
    current_timestamp: i64
) -> DateTime<Local> {
    let slot_duration_secs = 1.0 / slots_per_second;
    let slot_diff = slot as i64 - current_slot as i64;
    let timestamp = current_timestamp + (slot_diff as f64 * slot_duration_secs) as i64;

    let utc_time = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
    utc_time.with_timezone(&Local)
}

/// Calculates the starting slot for a given epoch
fn calculate_epoch_start_slot(
    target_epoch: u64,
    current_epoch: u64,
    current_absolute_slot: u64,
    current_slot_index: u64,
    slots_per_epoch: u64,
    first_normal_slot: u64,
) -> u64 {
    if target_epoch == current_epoch {
        // For current epoch, calculate from current position
        current_absolute_slot - current_slot_index
    } else if target_epoch == 0 {
        0
    } else if target_epoch * slots_per_epoch < first_normal_slot {
        // Very early epochs during warmup period
        target_epoch * slots_per_epoch / 2
    } else {
        // Normal epochs after warmup
        first_normal_slot + (target_epoch.saturating_sub(1)) * slots_per_epoch
    }
}

/// Formats a time difference in a human-readable way
///
/// Returns strings like "1d 2h 30m ago" for past times or "45m 12s" for future times
fn format_time_difference(current_timestamp: i64, target_timestamp: i64) -> String {
    let diff = target_timestamp - current_timestamp;
    let abs_diff = diff.abs();

    let days = abs_diff / 86400;
    let hours = (abs_diff % 86400) / 3600;
    let minutes = (abs_diff % 3600) / 60;
    let seconds = abs_diff % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 && days == 0 && hours == 0 {
        parts.push(format!("{}s", seconds));
    }

    if parts.is_empty() {
        "now".to_string()
    } else {
        let formatted = parts.join(" ");
        if diff < 0 {
            format!("{} ago", formatted)
        } else {
            formatted
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse and validate validator identity
    let identity = Pubkey::from_str(&args.identity)
        .map_err(|_| anyhow!("Invalid validator identity: {}", args.identity))?;

    // Connect to the specified cluster
    let cluster_url = get_cluster_url(&args.cluster)?;
    println!("🔍 Fetching leader schedule for validator: {}", identity);
    println!("🌐 Cluster: {} ({})", args.cluster, cluster_url);

    let client = RpcClient::new(cluster_url);

    // Get current network state for accurate timestamp calculations
    let current_slot = client
        .get_slot()
        .map_err(|e| anyhow!("Failed to get current slot: {}", e))?;
    let current_timestamp = Utc::now().timestamp();

    println!("🕒 Using current slot {} as time reference", current_slot);

    // Get current epoch information
    let epoch_info = client
        .get_epoch_info()
        .map_err(|e| anyhow!("Failed to get epoch info: {}", e))?;

    let target_epoch = args.epoch.unwrap_or(epoch_info.epoch);
    println!("📅 Target epoch: {}", target_epoch);

    // Get epoch schedule for slot calculations
    let epoch_schedule = client
        .get_epoch_schedule()
        .map_err(|e| anyhow!("Failed to get epoch schedule: {}", e))?;

    // Calculate the starting slot for the target epoch
    let epoch_start_slot = calculate_epoch_start_slot(
        target_epoch,
        epoch_info.epoch,
        epoch_info.absolute_slot,
        epoch_info.slot_index,
        epoch_schedule.slots_per_epoch,
        epoch_schedule.first_normal_slot,
    );

    println!(
        "🎯 Epoch {} starts at absolute slot: {}",
        target_epoch, epoch_start_slot
    );

    // Get leader schedule for the target epoch
    let leader_schedule = if args.epoch.is_some() {
        client.get_leader_schedule(Some(target_epoch))?
    } else {
        // For current epoch, use None to get the currently active schedule
        client.get_leader_schedule(None)?
    };

    match leader_schedule {
        Some(schedule) => {
            // Find slots assigned to our validator
            if let Some(slots) = schedule.get(&identity.to_string()) {
                if slots.is_empty() {
                    println!(
                        "ℹ️  No leader slots found for validator {} in epoch {}",
                        identity, target_epoch
                    );
                    return Ok(());
                }

                println!(
                    "✅ Found {} leader slots for validator {} in epoch {}",
                    slots.len(),
                    identity,
                    target_epoch
                );

                // Create table data with proper slot calculations and timing
                let mut table_data = Vec::new();
                for &relative_slot in slots {
                    let absolute_slot = epoch_start_slot + relative_slot as u64;
                    let time_local = slot_to_timestamp_local(
                        absolute_slot,
                        SLOTS_PER_SECOND,
                        current_slot,
                        current_timestamp
                    );
                    let slot_timestamp = time_local.timestamp();
                    let time_diff = format_time_difference(current_timestamp, slot_timestamp);

                    table_data.push(LeaderSlot {
                        epoch: target_epoch,
                        slot: absolute_slot,
                        time_local: time_local.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
                        time_diff,
                    });
                }

                // Sort by slot number for better readability
                table_data.sort_by_key(|entry| entry.slot);

                // Filter slots based on --all flag (show only upcoming slots by default)
                let filtered_data = if args.all {
                    table_data
                } else {
                    table_data.into_iter()
                        .filter(|slot| {
                            let slot_time = slot_to_timestamp_local(
                                slot.slot,
                                SLOTS_PER_SECOND,
                                current_slot,
                                current_timestamp
                            );
                            slot_time.timestamp() > current_timestamp
                        })
                        .collect()
                };

                if filtered_data.is_empty() {
                    if args.all {
                        println!(
                            "ℹ️  No leader slots found for validator {} in epoch {}",
                            identity, target_epoch
                        );
                    } else {
                        println!(
                            "ℹ️  No upcoming leader slots found for validator {} in epoch {} (use --all to see past slots)",
                            identity, target_epoch
                        );
                    }
                    return Ok(());
                }

                let slot_type = if args.all { "leader" } else { "upcoming leader" };
                println!(
                    "✅ Found {} {} slots for validator {} in epoch {}",
                    filtered_data.len(),
                    slot_type,
                    identity,
                    target_epoch
                );

                // Create table and display with custom header that includes current time
                let current_time_formatted = Local::now().format("%H:%M").to_string();
                let table = Table::new(&filtered_data);
                let table_string = table.to_string();

                // Replace the "Time Diff" header with the version that includes current time
                let custom_table = table_string.replace("Time Diff", &format!("Diff ({})", current_time_formatted));

                // Display the results in a formatted table
                println!("\n{}", custom_table);

                println!("\n📊 Total {} slots: {}", slot_type, filtered_data.len());
            } else {
                println!(
                    "ℹ️  Validator {} is not scheduled to be a leader in epoch {}",
                    identity, target_epoch
                );
            }
        }
        None => {
            println!("❌ No leader schedule found for epoch {}", target_epoch);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cluster_url() {
        assert!(get_cluster_url("mainnet").is_ok());
        assert!(get_cluster_url("testnet").is_ok());
        assert!(get_cluster_url("devnet").is_ok());
        assert!(get_cluster_url("invalid").is_err());
    }

    #[test]
    fn test_slot_to_timestamp() {
        let current_slot = 353285185u64;
        let current_timestamp = 1752511966i64;
        let timestamp = slot_to_timestamp_local(current_slot, SLOTS_PER_SECOND, current_slot, current_timestamp);
        // Should be very close to the current time since we're using the same slot
        assert!((timestamp.timestamp() - current_timestamp).abs() <= 2);
    }

    #[test]
    fn test_format_time_difference() {
        let current_time = 1000000000i64;

        // Test past times
        assert_eq!(format_time_difference(current_time, current_time - 3661), "1h 1m ago");
        assert_eq!(format_time_difference(current_time, current_time - 90061), "1d 1h 1m ago");
        assert_eq!(format_time_difference(current_time, current_time - 125), "2m 5s ago");

        // Test future times
        assert_eq!(format_time_difference(current_time, current_time + 3661), "1h 1m");
        assert_eq!(format_time_difference(current_time, current_time + 90061), "1d 1h 1m");
        assert_eq!(format_time_difference(current_time, current_time + 125), "2m 5s");

        // Test edge cases
        assert_eq!(format_time_difference(current_time, current_time), "now");
        assert_eq!(format_time_difference(current_time, current_time + 1), "1s");
        assert_eq!(format_time_difference(current_time, current_time - 1), "1s ago");
    }
}
