use crate::models::ModelStats;

/// Pricing per million tokens
#[derive(Debug, Clone, Copy)]
pub struct Pricing {
    pub input: f64,
    pub output: f64,
    pub cache_create: f64,
    pub cache_read: f64,
}

impl Pricing {
    pub const OPUS: Pricing = Pricing {
        input: 15.0,
        output: 75.0,
        cache_create: 18.75,
        cache_read: 1.50,
    };

    pub const SONNET: Pricing = Pricing {
        input: 3.0,
        output: 15.0,
        cache_create: 3.75,
        cache_read: 0.30,
    };

    pub const HAIKU: Pricing = Pricing {
        input: 0.25,
        output: 1.25,
        cache_create: 0.30,
        cache_read: 0.03,
    };
}

/// Get pricing for a model based on name
pub fn get_pricing(model: &str) -> Pricing {
    let model_lower = model.to_lowercase();
    if model_lower.contains("opus") {
        Pricing::OPUS
    } else if model_lower.contains("haiku") {
        Pricing::HAIKU
    } else {
        Pricing::SONNET
    }
}

/// Get tier name for display
pub fn get_tier(model: &str) -> &'static str {
    let model_lower = model.to_lowercase();
    if model_lower.contains("opus") {
        "Opus"
    } else if model_lower.contains("haiku") {
        "Haiku"
    } else {
        "Sonnet"
    }
}

/// Get tier color for display
pub fn get_tier_color(model: &str) -> ratatui::style::Color {
    use ratatui::style::Color;
    let model_lower = model.to_lowercase();
    if model_lower.contains("opus") {
        Color::Magenta
    } else if model_lower.contains("haiku") {
        Color::Green
    } else {
        Color::Cyan
    }
}

/// Calculate cost for a model's usage
pub fn calculate_cost(stats: &ModelStats) -> f64 {
    let pricing = get_pricing(&stats.model);
    let million = 1_000_000.0;

    (stats.input_tokens as f64 / million) * pricing.input
        + (stats.output_tokens as f64 / million) * pricing.output
        + (stats.cache_create_tokens as f64 / million) * pricing.cache_create
        + (stats.cache_read_tokens as f64 / million) * pricing.cache_read
}

/// Format token count with K/M suffix
pub fn format_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

/// Format cost as dollars
pub fn format_cost(cost: f64) -> String {
    if cost >= 100.0 {
        format!("${:.0}", cost)
    } else if cost >= 10.0 {
        format!("${:.1}", cost)
    } else {
        format!("${:.2}", cost)
    }
}
