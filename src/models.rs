use chrono::{DateTime, Utc};
use serde::Deserialize;

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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
pub struct PeriodStats {
    pub models: Vec<ModelStats>,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub total_calls: u64,
    pub session_count: usize,
    pub period_label: String,
}

/// Plan limits
#[derive(Debug, Clone)]
pub struct PlanLimits {
    pub name: &'static str,
    pub cost_limit: f64,
    pub message_limit: u64,
}

pub const PLANS: &[PlanLimits] = &[
    PlanLimits { name: "Pro", cost_limit: 18.0, message_limit: 250 },
    PlanLimits { name: "Max5", cost_limit: 35.0, message_limit: 1000 },
    PlanLimits { name: "Max20", cost_limit: 140.0, message_limit: 2000 },
];
