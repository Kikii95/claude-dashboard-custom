use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local, Timelike, Utc, DateTime};

use crate::calculator::{calculate_cost, calculate_entry_cost};
use crate::models::{CurrentBlockInfo, Entry, ModelStats, PeriodStats, RawEntry, SessionBlock};

/// Session duration in hours
const SESSION_HOURS: i64 = 5;

/// Get the Claude data directory
pub fn get_data_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

/// Find all JSONL files
pub fn find_jsonl_files(base: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_jsonl_files(&path));
            } else if path.extension().map_or(false, |e| e == "jsonl") {
                files.push(path);
            }
        }
    }
    files
}

/// Parse a single JSONL file
pub fn parse_file(path: &PathBuf) -> Vec<Entry> {
    let mut entries = Vec::new();

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return entries,
    };

    let reader = BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(raw) = serde_json::from_str::<RawEntry>(&line) {
            if let Ok(entry) = Entry::try_from(raw) {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Parse all JSONL files
pub fn parse_all() -> Result<Vec<Entry>> {
    let data_dir = get_data_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home dir"))?;

    if !data_dir.exists() {
        return Ok(Vec::new());
    }

    let files = find_jsonl_files(&data_dir);
    let mut all_entries: Vec<Entry> = files.iter().flat_map(parse_file).collect();

    // Sort by timestamp
    all_entries.sort_by_key(|e| e.timestamp);

    Ok(all_entries)
}

/// Round timestamp to the start of its hour (like claude-monitor)
fn round_to_hour(ts: DateTime<Utc>) -> DateTime<Utc> {
    ts.with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
}

/// Create session blocks from entries (5-hour blocks like claude-monitor)
pub fn create_blocks(entries: &[Entry]) -> Vec<SessionBlock> {
    if entries.is_empty() {
        return Vec::new();
    }

    let mut blocks: Vec<SessionBlock> = Vec::new();
    let session_duration = Duration::hours(SESSION_HOURS);

    for entry in entries {
        // Check if we need a new block
        let need_new_block = match blocks.last() {
            None => true,
            Some(current) => {
                // New block if entry is past current block's end time
                // OR if there's been a 5h+ gap since last entry
                entry.timestamp >= current.end_time
                    || (current.entries.last().map_or(true, |last| {
                        entry.timestamp - last.timestamp >= session_duration
                    }))
            }
        };

        if need_new_block {
            let start_time = round_to_hour(entry.timestamp);
            let end_time = start_time + session_duration;

            blocks.push(SessionBlock {
                start_time,
                end_time,
                is_active: false,
                entries: Vec::new(),
                stats: PeriodStats::default(),
            });
        }

        // Add entry to current block
        if let Some(block) = blocks.last_mut() {
            block.entries.push(entry.clone());
        }
    }

    // Mark active blocks and calculate stats
    let now = Utc::now();
    for block in &mut blocks {
        block.is_active = block.end_time > now && block.start_time <= now;
        block.stats = aggregate(&block.entries, "Block");
    }

    blocks
}

/// Find the current active block (or most recent if none active)
pub fn find_current_block(blocks: &[SessionBlock]) -> Option<&SessionBlock> {
    // First try to find an active block
    if let Some(active) = blocks.iter().find(|b| b.is_active) {
        return Some(active);
    }
    // Otherwise return the most recent
    blocks.last()
}

/// Get current block info for display
pub fn get_current_block_info(entries: &[Entry], plan_cost_limit: f64) -> CurrentBlockInfo {
    let blocks = create_blocks(entries);
    let now = Utc::now();

    // Find current or most recent block
    let current = find_current_block(&blocks);

    match current {
        Some(block) => {
            let secs_until_reset = (block.end_time - now).num_seconds().max(0);
            let is_active = block.is_active;

            // Calculate block stats
            let mut block_cost = 0.0;
            let mut block_tokens = 0u64;
            let block_calls = block.entries.len() as u64;

            for entry in &block.entries {
                block_cost += calculate_entry_cost(entry);
                block_tokens += entry.usage.total();
            }

            let usage_percent = if plan_cost_limit > 0.0 {
                (block_cost / plan_cost_limit) * 100.0
            } else {
                0.0
            };

            CurrentBlockInfo {
                block_start: Some(block.start_time),
                reset_time: Some(block.end_time),
                secs_until_reset,
                block_cost,
                block_tokens,
                block_calls,
                is_active,
                usage_percent,
            }
        }
        None => CurrentBlockInfo::default(),
    }
}

/// Filter entries for today only
pub fn filter_today(entries: &[Entry]) -> Vec<Entry> {
    let today = Local::now().date_naive();
    entries
        .iter()
        .filter(|e| e.timestamp.with_timezone(&Local).date_naive() == today)
        .cloned()
        .collect()
}

/// Filter entries for this week (Mon-Sun)
pub fn filter_this_week(entries: &[Entry]) -> Vec<Entry> {
    use chrono::Datelike;
    let now = Local::now();
    let today = now.date_naive();
    let days_since_monday = today.weekday().num_days_from_monday();
    let monday = today - Duration::days(days_since_monday as i64);

    entries
        .iter()
        .filter(|e| {
            let entry_date = e.timestamp.with_timezone(&Local).date_naive();
            entry_date >= monday && entry_date <= today
        })
        .cloned()
        .collect()
}

/// Filter entries for this month
pub fn filter_this_month(entries: &[Entry]) -> Vec<Entry> {
    use chrono::Datelike;
    let now = Local::now();
    let this_month = now.month();
    let this_year = now.year();

    entries
        .iter()
        .filter(|e| {
            let ts = e.timestamp.with_timezone(&Local);
            ts.month() == this_month && ts.year() == this_year
        })
        .cloned()
        .collect()
}

/// Aggregate entries into stats
pub fn aggregate(entries: &[Entry], label: &str) -> PeriodStats {
    let mut models_map: HashMap<String, ModelStats> = HashMap::new();
    let mut sessions: HashSet<String> = HashSet::new();

    for entry in entries {
        sessions.insert(entry.session_id.clone());

        let stats = models_map
            .entry(entry.model.clone())
            .or_insert_with(|| ModelStats::new(entry.model.clone()));
        stats.add(&entry.usage);
    }

    let mut models: Vec<ModelStats> = models_map.into_values().collect();
    // Sort by cost descending
    models.sort_by(|a, b| {
        calculate_cost(b)
            .partial_cmp(&calculate_cost(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_tokens: u64 = models.iter().map(|m| m.total_tokens()).sum();
    let total_calls: u64 = models.iter().map(|m| m.call_count).sum();
    let total_cost: f64 = models.iter().map(|m| calculate_cost(m)).sum();

    PeriodStats {
        models,
        total_tokens,
        total_cost,
        total_calls,
        session_count: sessions.len(),
        period_label: label.to_string(),
    }
}
