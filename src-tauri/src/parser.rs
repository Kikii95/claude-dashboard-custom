use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local, Timelike, Utc, DateTime};

use crate::calculator::{calculate_cost, calculate_entry_cost, calculate_entry_limit_cost, get_limit_tokens, get_tier};
use crate::models::{CurrentBlockInfo, Entry, ModelDistribution, ModelStats, PeriodStats, PlanLimits, RawEntry, SessionBlock};

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

/// Find the current active block ONLY
/// Returns None if no block is currently active (= after reset, usage is 0)
pub fn find_current_block(blocks: &[SessionBlock]) -> Option<&SessionBlock> {
    // Only return a block if it's currently active
    // If no active block, return None (user starts fresh after reset)
    blocks.iter().find(|b| b.is_active)
}

/// Get current block info for display with all metrics
pub fn get_current_block_info(entries: &[Entry], plan: &PlanLimits) -> CurrentBlockInfo {
    let now = Utc::now();

    // Use the proper block creation logic that handles gaps correctly
    let blocks = create_blocks(entries);
    let current_block = find_current_block(&blocks);

    // If no active block, return empty (session has reset)
    let block = match current_block {
        Some(b) => b,
        None => return CurrentBlockInfo::default(),
    };

    let block_start = block.start_time;
    let block_end = block.end_time;
    let secs_until_reset = (block_end - now).num_seconds().max(0);

    // Calculate metrics from THIS BLOCK's entries only
    let mut limit_cost = 0.0;
    let mut limit_tokens = 0u64;
    let limit_messages = block.entries.len() as u64;
    let mut real_cost = 0.0;
    let mut real_tokens = 0u64;

    for entry in &block.entries {
        limit_cost += calculate_entry_limit_cost(entry);
        limit_tokens += get_limit_tokens(entry);
        real_cost += calculate_entry_cost(entry);
        real_tokens += entry.usage.total();
    }

    // Calculate percentages
    let cost_percent = if plan.cost_limit > 0.0 {
        (limit_cost / plan.cost_limit) * 100.0
    } else {
        0.0
    };

    let tokens_percent = if plan.token_limit > 0 {
        (limit_tokens as f64 / plan.token_limit as f64) * 100.0
    } else {
        0.0
    };

    let messages_percent = if plan.message_limit > 0 {
        (limit_messages as f64 / plan.message_limit as f64) * 100.0
    } else {
        0.0
    };

    // Calculate burn rate
    let active_minutes = if block.entries.len() > 1 {
        let first_ts = block.entries.first().unwrap().timestamp;
        let last_ts = block.entries.last().unwrap().timestamp;
        let duration_mins = (last_ts - first_ts).num_seconds() as f64 / 60.0;
        duration_mins.max(1.0)
    } else {
        1.0
    };

    let tokens_per_min = limit_tokens as f64 / active_minutes;
    let cost_per_min = limit_cost / active_minutes;

    // Calculate predictions
    let tokens_remaining = if limit_tokens < plan.token_limit {
        plan.token_limit - limit_tokens
    } else {
        0
    };
    let cost_remaining = if limit_cost < plan.cost_limit {
        plan.cost_limit - limit_cost
    } else {
        0.0
    };

    let tokens_exhausted_at = if tokens_per_min > 0.0 && tokens_remaining > 0 {
        let mins_to_exhaust = tokens_remaining as f64 / tokens_per_min;
        Some(now + Duration::seconds((mins_to_exhaust * 60.0) as i64))
    } else if tokens_remaining == 0 {
        Some(now)
    } else {
        None
    };

    let cost_exhausted_at = if cost_per_min > 0.0 && cost_remaining > 0.0 {
        let mins_to_exhaust = cost_remaining / cost_per_min;
        Some(now + Duration::seconds((mins_to_exhaust * 60.0) as i64))
    } else if cost_remaining <= 0.0 {
        Some(now)
    } else {
        None
    };

    CurrentBlockInfo {
        block_start: Some(block_start),
        reset_time: Some(block_end),
        secs_until_reset,
        limit_cost,
        limit_tokens,
        limit_messages,
        real_cost,
        real_tokens,
        cost_percent,
        tokens_percent,
        messages_percent,
        tokens_per_min,
        cost_per_min,
        active_minutes,
        tokens_exhausted_at,
        cost_exhausted_at,
        is_active: block.is_active,
    }
}

/// Get model distribution for current active block only
pub fn get_model_distribution(entries: &[Entry]) -> Vec<ModelDistribution> {
    // Use the proper block system (same as get_current_block_info)
    let blocks = create_blocks(entries);
    let current_block = find_current_block(&blocks);

    let block = match current_block {
        Some(b) => b,
        None => return Vec::new(),
    };

    let mut dist_map: HashMap<String, (u64, u64, f64)> = HashMap::new(); // calls, tokens, cost
    let mut total_cost = 0.0;

    for entry in &block.entries {
        let tier = get_tier(&entry.model);
        let cost = calculate_entry_limit_cost(entry);
        let tokens = get_limit_tokens(entry);
        total_cost += cost;

        let e = dist_map.entry(tier.to_string()).or_insert((0, 0, 0.0));
        e.0 += 1;
        e.1 += tokens;
        e.2 += cost;
    }

    let mut result: Vec<ModelDistribution> = dist_map
        .into_iter()
        .map(|(tier, (calls, tokens, cost))| {
            let percent = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            ModelDistribution {
                model: tier.clone(),
                tier,
                calls,
                tokens,
                cost,
                percent,
            }
        })
        .collect();

    // Sort by cost descending
    result.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(std::cmp::Ordering::Equal));
    result
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
