use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local, Utc};

use crate::calculator::{calculate_cost, calculate_entry_cost};
use crate::models::{Entry, ModelStats, PeriodStats, RateLimitInfo, RawEntry};

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

/// Filter entries by time period
#[allow(dead_code)]
pub fn filter_by_days(entries: &[Entry], days: i64) -> Vec<Entry> {
    let cutoff = Utc::now() - Duration::days(days);
    entries
        .iter()
        .filter(|e| e.timestamp >= cutoff)
        .cloned()
        .collect()
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

/// Get stats for different periods
#[allow(dead_code)]
pub fn get_all_stats(entries: &[Entry]) -> (PeriodStats, PeriodStats, PeriodStats, PeriodStats) {
    let today = aggregate(&filter_today(entries), "Today");
    let week = aggregate(&filter_this_week(entries), "This Week");
    let month = aggregate(&filter_this_month(entries), "This Month");
    let all_time = aggregate(entries, "All Time");

    (today, week, month, all_time)
}

/// 5-hour window constant
const RATE_LIMIT_WINDOW_HOURS: i64 = 5;

/// Filter entries within the 5-hour rate limit window
pub fn filter_rate_limit_window(entries: &[Entry]) -> Vec<Entry> {
    let cutoff = Utc::now() - Duration::hours(RATE_LIMIT_WINDOW_HOURS);
    entries
        .iter()
        .filter(|e| e.timestamp >= cutoff)
        .cloned()
        .collect()
}

/// Calculate rate limit info for the 5-hour rolling window
pub fn calculate_rate_limit(entries: &[Entry], plan_cost_limit: f64) -> RateLimitInfo {
    let now = Utc::now();
    let window_entries = filter_rate_limit_window(entries);

    if window_entries.is_empty() {
        return RateLimitInfo::default();
    }

    // Calculate totals for the window
    let mut window_cost = 0.0;
    let mut window_tokens = 0u64;
    let window_calls = window_entries.len() as u64;

    for entry in &window_entries {
        window_cost += calculate_entry_cost(entry);
        window_tokens += entry.usage.total();
    }

    // Find oldest entry
    let oldest = window_entries.first().map(|e| e.timestamp);

    // Calculate when oldest entry expires (partial reset)
    let secs_until_partial_reset = oldest.map(|ts| {
        let expires_at = ts + Duration::hours(RATE_LIMIT_WINDOW_HOURS);
        (expires_at - now).num_seconds().max(0)
    });

    // Calculate cost that will be freed at partial reset
    // Group entries by minute to find first batch
    let partial_reset_cost = if let Some(oldest_ts) = oldest {
        let first_minute = oldest_ts + Duration::minutes(1);
        window_entries
            .iter()
            .take_while(|e| e.timestamp < first_minute)
            .map(calculate_entry_cost)
            .sum()
    } else {
        0.0
    };

    // Full reset = when all entries expire (5h from now if no new activity)
    let newest = window_entries.last().map(|e| e.timestamp);
    let secs_until_full_reset = newest.map(|ts| {
        let expires_at = ts + Duration::hours(RATE_LIMIT_WINDOW_HOURS);
        (expires_at - now).num_seconds().max(0)
    });

    // Estimate if rate limited (cost > 80% of limit)
    let is_limited = window_cost >= plan_cost_limit * 0.8;

    RateLimitInfo {
        window_cost,
        window_calls,
        window_tokens,
        secs_until_partial_reset,
        partial_reset_cost,
        secs_until_full_reset,
        oldest_entry: oldest,
        is_limited,
    }
}
