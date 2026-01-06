// Types matching Rust backend

export interface ModelStats {
  model: string;
  input_tokens: number;
  output_tokens: number;
  cache_create_tokens: number;
  cache_read_tokens: number;
  call_count: number;
}

export interface PeriodStats {
  models: ModelStats[];
  total_tokens: number;
  total_cost: number;
  total_calls: number;
  session_count: number;
  period_label: string;
}

export interface PlanLimits {
  name: string;
  token_limit: number;
  cost_limit: number;
  message_limit: number;
}

export interface CurrentBlockInfo {
  // Block timing
  block_start: string | null;
  reset_time: string | null;
  secs_until_reset: number;

  // LIMIT metrics (what counts towards rate limit - input + output only)
  limit_cost: number;
  limit_tokens: number;
  limit_messages: number;

  // REAL metrics (actual usage including cache)
  real_cost: number;
  real_tokens: number;

  // Percentages (based on limit metrics)
  cost_percent: number;
  tokens_percent: number;
  messages_percent: number;

  // Burn rate
  tokens_per_min: number;
  cost_per_min: number;
  active_minutes: number;

  // Predictions
  tokens_exhausted_at: string | null;
  cost_exhausted_at: string | null;

  // Status
  is_active: boolean;
}

export interface ModelDistribution {
  model: string;
  tier: string;
  calls: number;
  tokens: number;
  cost: number;
  percent: number;
}

export interface DashboardData {
  current_block: CurrentBlockInfo;
  today: PeriodStats;
  week: PeriodStats;
  month: PeriodStats;
  selected_plan: PlanLimits;
  model_distribution: ModelDistribution[];
  warnings: string[];
}
