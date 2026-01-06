import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DashboardData, PlanLimits, ModelDistribution } from "./types";
import { themes, themeKeys, applyTheme, getStoredTheme, storeTheme } from "./themes";

// Format helpers
const formatTokens = (count: number): string => {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`;
  return count.toString();
};

const formatCost = (cost: number): string => {
  if (cost >= 100) return `$${cost.toFixed(0)}`;
  if (cost >= 10) return `$${cost.toFixed(1)}`;
  return `$${cost.toFixed(2)}`;
};

const formatDuration = (secs: number): string => {
  if (secs <= 0) return "now";
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (hours > 0) return `${hours}h ${mins.toString().padStart(2, "0")}m`;
  if (mins > 0) return `${mins}m ${s.toString().padStart(2, "0")}s`;
  return `${s}s`;
};

const formatTime = (isoString: string | null): string => {
  if (!isoString) return "N/A";
  const date = new Date(isoString);
  return date.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
};

const getTierBadge = (tier: string) => {
  if (tier === "Opus") return { name: "Opus", class: "badge-opus" };
  if (tier === "Haiku") return { name: "Haiku", class: "badge-haiku" };
  return { name: "Sonnet", class: "badge-sonnet" };
};

// Progress Bar component
const ProgressBar = ({
  value,
  max,
  label,
  accentClass,
  showValues,
}: {
  value: number;
  max: number;
  label: string;
  accentClass: string;
  showValues?: boolean;
}) => {
  const percent = max > 0 ? (value / max) * 100 : 0;
  const isOverflow = percent > 100;
  const displayPercent = Math.min(percent, 100);

  return (
    <div className="space-y-1">
      <div className="flex justify-between text-sm">
        <span className="text-secondary">{label}</span>
        <div className="flex items-center gap-3">
          {showValues && (
            <span className="font-mono text-primary text-xs">
              {typeof value === "number" && value < 1000 ? value : formatTokens(value)} / {typeof max === "number" && max < 1000 ? max : formatTokens(max)}
            </span>
          )}
          <span className={`font-mono font-bold ${isOverflow ? "text-error" : "text-accent-1"}`}>
            {percent.toFixed(1)}%
          </span>
        </div>
      </div>
      <div className="progress-bar">
        <div
          className={`progress-fill ${accentClass} ${isOverflow ? "animate-pulse" : ""}`}
          style={{ width: `${displayPercent}%` }}
        />
      </div>
    </div>
  );
};

// Main Stat (big numbers like claude-dashboard)
const MainStat = ({
  value,
  max,
  label,
  icon,
}: {
  value: string;
  max: string;
  label: string;
  icon: string;
}) => (
  <div className="text-center">
    <div className="text-xs text-secondary uppercase tracking-wider mb-1 flex items-center justify-center gap-1">
      <span>{icon}</span> {label}
    </div>
    <div className="text-3xl font-bold font-mono text-gradient">{value}</div>
    <div className="text-sm text-secondary">/ {max}</div>
  </div>
);

// Mini Stat component for secondary info
const MiniStat = ({ label, value, color }: { label: string; value: string; color?: string }) => (
  <div className="flex justify-between items-center py-1">
    <span className="text-secondary text-sm">{label}</span>
    <span className={`font-mono text-sm ${color || "text-primary"}`}>{value}</span>
  </div>
);

// Small Card for secondary metrics
const InfoCard = ({
  title,
  icon,
  children,
}: {
  title: string;
  icon: string;
  children: React.ReactNode;
}) => (
  <div className="card">
    <h3 className="text-xs font-semibold text-secondary mb-2 flex items-center gap-1 uppercase tracking-wider">
      <span>{icon}</span> {title}
    </h3>
    {children}
  </div>
);

// Model Distribution Bar
const ModelDistBar = ({ dist }: { dist: ModelDistribution }) => {
  const tier = getTierBadge(dist.tier);
  return (
    <div className="flex items-center gap-2 py-1">
      <span className={`badge ${tier.class} w-14 text-center text-xs`}>{tier.name}</span>
      <div className="flex-1 progress-bar h-1.5">
        <div
          className="progress-fill accent-2"
          style={{ width: `${Math.min(dist.percent, 100)}%` }}
        />
      </div>
      <span className="font-mono text-xs text-accent-1 w-12 text-right">{formatCost(dist.cost)}</span>
    </div>
  );
};

// Warning Banner
const WarningBanner = ({ warnings }: { warnings: string[] }) => {
  if (warnings.length === 0) return null;
  return (
    <div className="bg-error/20 border border-error/50 rounded-lg p-3 mb-4">
      {warnings.map((w, i) => (
        <div key={i} className="text-error text-sm font-medium">
          {w}
        </div>
      ))}
    </div>
  );
};

// Theme Selector component
const ThemeSelector = ({
  currentTheme,
  onThemeChange,
  isOpen,
  onToggle,
}: {
  currentTheme: string;
  onThemeChange: (theme: string) => void;
  isOpen: boolean;
  onToggle: () => void;
}) => (
  <div className="relative">
    <button
      onClick={onToggle}
      className="theme-btn flex items-center gap-2 px-3 py-2"
      title="Change theme"
    >
      <span>{themes[currentTheme]?.icon || "🎨"}</span>
      <span className="text-sm hidden sm:inline">{themes[currentTheme]?.name}</span>
    </button>

    {isOpen && (
      <>
        <div className="fixed inset-0 z-40" onClick={onToggle} />
        <div className="absolute right-0 top-full mt-2 z-50 card p-3 min-w-[280px] shadow-xl">
          <div className="text-sm font-semibold text-secondary mb-3">Select Theme</div>
          <div className="theme-grid">
            {themeKeys.map((key) => {
              const theme = themes[key];
              return (
                <button
                  key={key}
                  onClick={() => {
                    onThemeChange(key);
                    onToggle();
                  }}
                  className={`theme-btn flex flex-col items-center gap-1 p-2 ${
                    currentTheme === key ? "active" : ""
                  }`}
                >
                  <span className="text-xl">{theme.icon}</span>
                  <span className="text-xs">{theme.name}</span>
                </button>
              );
            })}
          </div>
        </div>
      </>
    )}
  </div>
);

function App() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [plans, setPlans] = useState<PlanLimits[]>([]);
  const [planIndex, setPlanIndex] = useState(1); // Default to Max5
  const [error, setError] = useState<string | null>(null);
  const [countdown, setCountdown] = useState(0);
  const [currentTheme, setCurrentTheme] = useState(getStoredTheme());
  const [themeMenuOpen, setThemeMenuOpen] = useState(false);

  useEffect(() => {
    applyTheme(currentTheme);
  }, [currentTheme]);

  const handleThemeChange = (theme: string) => {
    setCurrentTheme(theme);
    storeTheme(theme);
    applyTheme(theme);
  };

  useEffect(() => {
    invoke<PlanLimits[]>("get_available_plans")
      .then(setPlans)
      .catch((e) => console.error("Failed to get plans:", e));
  }, []);

  const fetchData = useCallback(async () => {
    try {
      const result = await invoke<DashboardData>("get_dashboard_data", { planIndex });
      setData(result);
      setCountdown(result.current_block.secs_until_reset);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [planIndex]);

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, [fetchData]);

  useEffect(() => {
    if (countdown <= 0) return;
    const timer = setInterval(() => {
      setCountdown((c) => Math.max(0, c - 1));
    }, 1000);
    return () => clearInterval(timer);
  }, [countdown > 0]);

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center p-8">
        <div className="card max-w-md text-center">
          <div className="text-4xl mb-4">⚠️</div>
          <h2 className="text-xl font-bold text-error mb-2">Error</h2>
          <p className="text-secondary">{error}</p>
          <button
            onClick={fetchData}
            className="mt-4 px-4 py-2 bg-accent-1 hover:opacity-80 rounded-lg transition-all"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin text-4xl mb-4">⚙️</div>
          <p className="text-secondary">Loading dashboard...</p>
        </div>
      </div>
    );
  }

  const { current_block, today, week, month, selected_plan, model_distribution, warnings } = data;

  return (
    <div className="min-h-screen p-4 space-y-4">
      {/* Header */}
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-gradient">Claude Dashboard</h1>
          <p className="text-secondary text-xs">5h Rate Limit Tracker</p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={planIndex}
            onChange={(e) => setPlanIndex(Number(e.target.value))}
            className="bg-secondary border border-white/10 rounded-lg px-2 py-1.5 text-sm focus:outline-none focus:border-accent-1 text-primary"
          >
            {plans.map((plan, i) => (
              <option key={plan.name} value={i}>{plan.name}</option>
            ))}
          </select>
          <ThemeSelector
            currentTheme={currentTheme}
            onThemeChange={handleThemeChange}
            isOpen={themeMenuOpen}
            onToggle={() => setThemeMenuOpen(!themeMenuOpen)}
          />
          <button onClick={fetchData} className="theme-btn p-2" title="Refresh">🔄</button>
        </div>
      </header>

      {/* Warnings */}
      <WarningBanner warnings={warnings} />

      {/* ═══════════════════════════════════════════════════════════════════
          ZONE PRINCIPALE — Métriques claude-dashboard (référence précise)
          ═══════════════════════════════════════════════════════════════════ */}
      <section className="card glow">
        {/* Header with status + countdown */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div
              className={`w-3 h-3 rounded-full ${
                current_block.is_active ? "bg-success animate-pulse" : "bg-secondary opacity-50"
              }`}
            />
            <div>
              <h2 className="text-lg font-bold">Current Block</h2>
              <span className="text-xs text-secondary">
                {current_block.block_start ? `${formatTime(current_block.block_start)} → ${formatTime(current_block.reset_time)}` : "No active block"}
              </span>
            </div>
          </div>
          <div className="text-right">
            <div className="text-3xl font-mono font-bold text-accent-1">
              {formatDuration(countdown)}
            </div>
            <div className="text-xs text-secondary">until reset</div>
          </div>
        </div>

        {/* Main Stats - Like claude-dashboard */}
        <div className="grid grid-cols-3 gap-6 mb-6">
          <MainStat
            icon="💰"
            label="Cost"
            value={formatCost(current_block.limit_cost)}
            max={formatCost(selected_plan.cost_limit)}
          />
          <MainStat
            icon="🎯"
            label="Tokens"
            value={formatTokens(current_block.limit_tokens)}
            max={formatTokens(selected_plan.token_limit)}
          />
          <MainStat
            icon="💬"
            label="Messages"
            value={current_block.limit_messages.toString()}
            max={selected_plan.message_limit.toString()}
          />
        </div>

        {/* Progress Bars */}
        <div className="space-y-3">
          <ProgressBar
            value={current_block.limit_cost}
            max={selected_plan.cost_limit}
            accentClass="accent-1"
            label="Cost"
          />
          <ProgressBar
            value={current_block.limit_tokens}
            max={selected_plan.token_limit}
            accentClass="accent-2"
            label="Tokens"
          />
          <ProgressBar
            value={current_block.limit_messages}
            max={selected_plan.message_limit}
            accentClass="accent-3"
            label="Messages"
          />
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════
          ZONE SECONDAIRE — Infos supplémentaires (bonus)
          ═══════════════════════════════════════════════════════════════════ */}
      <div className="grid grid-cols-4 gap-3">
        {/* Burn Rate */}
        <InfoCard title="Burn Rate" icon="🔥">
          <MiniStat label="Tokens/min" value={current_block.tokens_per_min.toFixed(0)} color="text-accent-2" />
          <MiniStat label="Cost/min" value={formatCost(current_block.cost_per_min)} color="text-accent-1" />
          <MiniStat label="Active" value={`${current_block.active_minutes.toFixed(0)}m`} color="text-success" />
        </InfoCard>

        {/* Predictions */}
        <InfoCard title="Predictions" icon="🔮">
          <MiniStat
            label="Tokens out"
            value={current_block.tokens_exhausted_at ? formatTime(current_block.tokens_exhausted_at) : "Safe ✓"}
            color={current_block.tokens_exhausted_at ? "text-warning" : "text-success"}
          />
          <MiniStat
            label="Cost out"
            value={current_block.cost_exhausted_at ? formatTime(current_block.cost_exhausted_at) : "Safe ✓"}
            color={current_block.cost_exhausted_at ? "text-warning" : "text-success"}
          />
        </InfoCard>

        {/* Real Usage (with cache) */}
        <InfoCard title="Real Usage" icon="📈">
          <MiniStat label="Real cost" value={formatCost(current_block.real_cost)} color="text-accent-1" />
          <MiniStat label="Real tokens" value={formatTokens(current_block.real_tokens)} color="text-accent-2" />
          <MiniStat
            label="Cache saved"
            value={formatCost(Math.max(0, current_block.real_cost - current_block.limit_cost))}
            color="text-success"
          />
        </InfoCard>

        {/* Model Distribution */}
        <InfoCard title="Models" icon="🤖">
          {model_distribution.length > 0 ? (
            model_distribution.map((dist) => (
              <ModelDistBar key={dist.tier} dist={dist} />
            ))
          ) : (
            <div className="text-xs text-secondary">No data</div>
          )}
        </InfoCard>
      </div>

      {/* Period Stats */}
      <div className="grid grid-cols-3 gap-3">
        {[today, week, month].map((period) => (
          <div key={period.period_label} className="card">
            <h3 className="text-xs font-semibold text-secondary mb-2 uppercase tracking-wider">
              {period.period_label}
            </h3>
            <div className="grid grid-cols-2 gap-x-4">
              <MiniStat label="Cost" value={formatCost(period.total_cost)} color="text-accent-1" />
              <MiniStat label="Tokens" value={formatTokens(period.total_tokens)} color="text-accent-2" />
              <MiniStat label="Calls" value={period.total_calls.toString()} color="text-success" />
              <MiniStat label="Sessions" value={period.session_count.toString()} />
            </div>
          </div>
        ))}
      </div>

      {/* Model Breakdown Today - Collapsible detail */}
      {today.models.length > 0 && (
        <details className="card">
          <summary className="text-xs font-semibold text-secondary cursor-pointer hover:text-primary transition-colors">
            📊 Model Details (Today) — {today.models.length} model(s)
          </summary>
          <div className="mt-3 space-y-2">
            {today.models.map((model) => {
              const tier = getTierBadge(
                model.model.toLowerCase().includes("opus") ? "Opus" :
                model.model.toLowerCase().includes("haiku") ? "Haiku" : "Sonnet"
              );
              const totalTokens = model.input_tokens + model.output_tokens + model.cache_create_tokens + model.cache_read_tokens;
              return (
                <div
                  key={model.model}
                  className="flex items-center justify-between py-1 border-b border-white/5 last:border-0"
                >
                  <div className="flex items-center gap-2">
                    <span className={`badge ${tier.class}`}>{tier.name}</span>
                    <span className="text-xs text-primary truncate max-w-[180px]">{model.model}</span>
                  </div>
                  <div className="flex items-center gap-3 text-xs font-mono">
                    <span className="text-accent-2">{formatTokens(totalTokens)}</span>
                    <span className="text-secondary">{model.call_count} calls</span>
                  </div>
                </div>
              );
            })}
          </div>
        </details>
      )}

      {/* Footer */}
      <footer className="text-center text-xs text-secondary opacity-50">
        Claude Dashboard v0.8.0 • {selected_plan.name} • {themes[currentTheme]?.name}
      </footer>
    </div>
  );
}

export default App;
