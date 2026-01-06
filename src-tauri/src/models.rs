use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Raw usage data from JSONL
#[derive(Debug, Deserialize)]
pub struct RawEntry {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_input_tokens + self.cache_read_input_tokens
    }
}

/// Parsed entry with all required fields
#[derive(Debug, Clone)]
pub struct Entry {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub model: String,
    pub usage: Usage,
}

impl TryFrom<RawEntry> for Entry {
    type Error = ();

    fn try_from(raw: RawEntry) -> Result<Self, Self::Error> {
        let message = raw.message.ok_or(())?;
        let usage = message.usage.ok_or(())?;
        let model = message.model.ok_or(())?;

        // Skip entries with no tokens
        if usage.total() == 0 {
            return Err(());
        }

        Ok(Entry {
            timestamp: raw.timestamp,
            session_id: raw.session_id.unwrap_or_else(|| "unknown".into()),
            model,
            usage,
        })
    }
}

/// Aggregated stats per model
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelStats {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_create_tokens: u64,
    pub cache_read_tokens: u64,
    pub call_count: u64,
}

impl ModelStats {
    pub fn new(model: String) -> Self {
        Self { model, ..Default::default() }
    }

    pub fn add(&mut self, usage: &Usage) {
        self.input_tokens += usage.input_tokens;
        self.output_tokens += usage.output_tokens;
        self.cache_create_tokens += usage.cache_creation_input_tokens;
        self.cache_read_tokens += usage.cache_read_input_tokens;
        self.call_count += 1;
    }

    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_create_tokens + self.cache_read_tokens
    }
}

/// Stats for a time period
#[derive(Debug, Clone, Default, Serialize)]
pub struct PeriodStats {
    pub models: Vec<ModelStats>,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub total_calls: u64,
    pub session_count: usize,
    pub period_label: String,
}

/// Plan limits (from claude-monitor/core/plans.py)
#[derive(Debug, Clone, Serialize)]
pub struct PlanLimits {
    pub name: String,
    pub token_limit: u64,
    pub cost_limit: f64,
    pub message_limit: u64,
}

pub fn get_plans() -> Vec<PlanLimits> {
    vec![
        PlanLimits { name: "Pro".into(), token_limit: 19_000, cost_limit: 18.0, message_limit: 250 },
        PlanLimits { name: "Max5".into(), token_limit: 88_000, cost_limit: 35.0, message_limit: 1_000 },
        PlanLimits { name: "Max20".into(), token_limit: 220_000, cost_limit: 140.0, message_limit: 2_000 },
    ]
}

pub static PLANS: std::sync::LazyLock<Vec<PlanLimits>> = std::sync::LazyLock::new(get_plans);

/// A 5-hour session block (like claude-monitor)
#[derive(Debug, Clone)]
pub struct SessionBlock {
    /// Block start time (rounded to hour)
    pub start_time: DateTime<Utc>,
    /// Block end time (start + 5h = reset time)
    pub end_time: DateTime<Utc>,
    /// Is this the currently active block?
    pub is_active: bool,
    /// Entries in this block
    pub entries: Vec<Entry>,
    /// Stats for this block
    pub stats: PeriodStats,
}

/// Current block info for display
#[derive(Debug, Clone, Default, Serialize)]
pub struct CurrentBlockInfo {
    /// Block start time
    pub block_start: Option<DateTime<Utc>>,
    /// Block end time (= reset time!)
    pub reset_time: Option<DateTime<Utc>>,
    /// Seconds until reset
    pub secs_until_reset: i64,

    // === LIMIT METRICS (what counts towards rate limit) ===
    /// Cost towards limit (input + output only)
    pub limit_cost: f64,
    /// Tokens towards limit (input + output only)
    pub limit_tokens: u64,
    /// Messages count
    pub limit_messages: u64,

    // === REAL METRICS (actual usage including cache) ===
    /// Real total cost (all tokens)
    pub real_cost: f64,
    /// Real total tokens (all tokens)
    pub real_tokens: u64,

    // === PERCENTAGES ===
    pub cost_percent: f64,
    pub tokens_percent: f64,
    pub messages_percent: f64,

    // === BURN RATE ===
    /// Tokens per minute
    pub tokens_per_min: f64,
    /// Cost per minute
    pub cost_per_min: f64,
    /// Minutes active in this block
    pub active_minutes: f64,

    // === PREDICTIONS ===
    /// Predicted time when tokens run out (timestamp)
    pub tokens_exhausted_at: Option<DateTime<Utc>>,
    /// Predicted time when cost limit hit (timestamp)
    pub cost_exhausted_at: Option<DateTime<Utc>>,

    /// Is currently active (within 5h window)?
    pub is_active: bool,
}

/// Model distribution info
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelDistribution {
    pub model: String,
    pub tier: String,
    pub calls: u64,
    pub tokens: u64,
    pub cost: f64,
    pub percent: f64,
}

/// Dashboard data sent to frontend
#[derive(Debug, Clone, Serialize)]
pub struct DashboardData {
    pub current_block: CurrentBlockInfo,
    pub today: PeriodStats,
    pub week: PeriodStats,
    pub month: PeriodStats,
    pub selected_plan: PlanLimits,
    /// Model distribution in current block
    pub model_distribution: Vec<ModelDistribution>,
    /// Warning flags
    pub warnings: Vec<String>,
}
