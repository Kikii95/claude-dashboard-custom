use std::time::{Duration, Instant};

use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Padding, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::calculator::{calculate_cost, format_cost, format_duration, format_tokens, get_tier, get_tier_color};
use crate::models::{CurrentBlockInfo, Entry, PeriodStats, PlanLimits, PLANS};
use crate::parser::{aggregate, get_current_block_info, filter_this_month, filter_this_week, filter_today, parse_all};

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

pub struct App {
    pub entries: Vec<Entry>,
    pub today: PeriodStats,
    pub week: PeriodStats,
    pub month: PeriodStats,
    pub all_time: PeriodStats,
    pub current_block: CurrentBlockInfo,
    pub selected_plan: usize,
    pub selected_period: usize,
    pub last_refresh: Instant,
    pub last_data_refresh: Instant,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            entries: Vec::new(),
            today: PeriodStats::default(),
            week: PeriodStats::default(),
            month: PeriodStats::default(),
            all_time: PeriodStats::default(),
            current_block: CurrentBlockInfo::default(),
            selected_plan: 0,
            selected_period: 0,
            last_refresh: Instant::now(),
            last_data_refresh: Instant::now(),
            should_quit: false,
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        if let Ok(entries) = parse_all() {
            self.entries = entries;
            self.today = aggregate(&filter_today(&self.entries), "Today");
            self.week = aggregate(&filter_this_week(&self.entries), "This Week");
            self.month = aggregate(&filter_this_month(&self.entries), "This Month");
            self.all_time = aggregate(&self.entries, "All Time");
            self.current_block = get_current_block_info(&self.entries, self.current_plan().cost_limit);
            self.last_data_refresh = Instant::now();
        }
        self.last_refresh = Instant::now();
    }

    pub fn maybe_refresh(&mut self) {
        // Data refresh every 5 seconds
        if self.last_data_refresh.elapsed() >= Duration::from_secs(5) {
            self.refresh();
        }
        // UI refresh every second for countdown
        if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
            self.current_block = get_current_block_info(&self.entries, self.current_plan().cost_limit);
            self.last_refresh = Instant::now();
        }
    }

    pub fn current_stats(&self) -> &PeriodStats {
        match self.selected_period {
            0 => &self.today,
            1 => &self.week,
            2 => &self.month,
            _ => &self.all_time,
        }
    }

    pub fn current_plan(&self) -> &PlanLimits {
        &PLANS[self.selected_plan]
    }

    pub fn next_period(&mut self) {
        self.selected_period = (self.selected_period + 1) % 4;
    }

    pub fn prev_period(&mut self) {
        self.selected_period = if self.selected_period == 0 { 3 } else { self.selected_period - 1 };
    }

    pub fn next_plan(&mut self) {
        self.selected_plan = (self.selected_plan + 1) % PLANS.len();
        self.current_block = get_current_block_info(&self.entries, self.current_plan().cost_limit);
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
            ])
            .split(area);

        self.draw_header(frame, main_chunks[0]);
        self.draw_content(frame, main_chunks[1]);
        self.draw_footer(frame, main_chunks[2]);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let periods = ["Today", "Week", "Month", "All"];
        let tabs = Tabs::new(periods)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" ⚡ Claude Dashboard ")
                    .title_style(Style::default().fg(Color::Yellow).bold()),
            )
            .select(self.selected_period)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(Style::default().fg(Color::Cyan).bold().underlined());

        frame.render_widget(tabs, area);
    }

    fn draw_content(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        self.draw_left_panel(frame, chunks[0]);
        self.draw_models_panel(frame, chunks[1]);
    }

    fn draw_left_panel(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Current Block (MAIN FEATURE)
                Constraint::Length(6),  // Period Summary
                Constraint::Min(3),     // By tier
            ])
            .split(area);

        self.draw_current_block(frame, chunks[0]);
        self.draw_summary(frame, chunks[1]);
        self.draw_tier_costs(frame, chunks[2]);
    }

    fn draw_current_block(&self, frame: &mut Frame, area: Rect) {
        let cb = &self.current_block;
        let plan = self.current_plan();

        // Status
        let is_over = cb.usage_percent >= 100.0;
        let status_color = if is_over {
            Color::Red
        } else if cb.usage_percent >= 80.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        let status_icon = if is_over { "🔴" } else if cb.usage_percent >= 80.0 { "🟡" } else { "🟢" };

        // Format reset time in local timezone
        let reset_str = cb.reset_time
            .map(|t| t.with_timezone(&Local).format("%Hh%M").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let block_start_str = cb.block_start
            .map(|t| t.with_timezone(&Local).format("%Hh%M").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let mut lines = vec![
            // Status line
            Line::from(vec![
                Span::styled(format!(" {} ", status_icon), Style::default()),
                Span::styled(format!("{:.0}%", cb.usage_percent), Style::default().fg(status_color).bold()),
                Span::styled(format!(" of {} limit", plan.name), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            // Reset time (THE KEY INFO!)
            Line::from(vec![
                Span::styled(" 🔄 Reset: ", Style::default().fg(Color::White)),
                Span::styled(&reset_str, Style::default().fg(Color::Cyan).bold()),
                Span::styled(format!(" (in {})", format_duration(cb.secs_until_reset)), Style::default().fg(Color::DarkGray)),
            ]),
            // Block info
            Line::from(vec![
                Span::styled(" 📍 Block: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} → {}", block_start_str, reset_str), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            // Usage in this block
            Line::from(vec![
                Span::styled(" 💵 Cost:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_cost(cb.block_cost), Style::default().fg(Color::Yellow).bold()),
                Span::styled(format!(" / {}", format_cost(plan.cost_limit)), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(" 📊 Tokens: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_tokens(cb.block_tokens), Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(vec![
                Span::styled(" 📞 Calls:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", cb.block_calls), Style::default().fg(Color::Blue).bold()),
            ]),
        ];

        let title = if is_over {
            " ⚠️  LIMIT REACHED "
        } else if cb.is_active {
            " ⏰ Current 5h Block "
        } else {
            " 💤 No Active Block "
        };

        let border_color = if is_over { Color::Red } else { Color::Cyan };

        let panel = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title)
                    .title_style(Style::default().fg(border_color).bold()),
            );

        frame.render_widget(panel, area);
    }

    fn draw_summary(&self, frame: &mut Frame, area: Rect) {
        let stats = self.current_stats();

        let summary_text = vec![
            Line::from(vec![
                Span::styled(" 💰 Cost:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_cost(stats.total_cost), Style::default().fg(Color::Yellow).bold()),
            ]),
            Line::from(vec![
                Span::styled(" 📊 Tokens:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_tokens(stats.total_tokens), Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(vec![
                Span::styled(" 📞 Calls:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", stats.total_calls), Style::default().fg(Color::Blue).bold()),
            ]),
        ];

        let summary = Paragraph::new(summary_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(format!(" {} [←/→] ", stats.period_label))
                    .title_style(Style::default().fg(Color::Green).bold())
                    .padding(Padding::vertical(1)),
            );

        frame.render_widget(summary, area);
    }

    fn draw_tier_costs(&self, frame: &mut Frame, area: Rect) {
        let stats = self.current_stats();

        let mut tier_costs: Vec<(&str, f64, Color)> = Vec::new();
        for model in &stats.models {
            let tier = get_tier(&model.model);
            let cost = calculate_cost(model);
            let color = get_tier_color(&model.model);

            if let Some(existing) = tier_costs.iter_mut().find(|(t, _, _)| *t == tier) {
                existing.1 += cost;
            } else {
                tier_costs.push((tier, cost, color));
            }
        }

        let tier_lines: Vec<Line> = tier_costs
            .iter()
            .map(|(tier, cost, color)| {
                Line::from(vec![
                    Span::styled(format!(" {} ", tier), Style::default().fg(*color).bold()),
                    Span::styled(format_cost(*cost), Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let tier_block = Paragraph::new(tier_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(format!(" By Tier [p: {}] ", self.current_plan().name)),
            );

        frame.render_widget(tier_block, area);
    }

    fn draw_models_panel(&self, frame: &mut Frame, area: Rect) {
        let stats = self.current_stats();

        let header = Row::new(vec!["Model", "Tier", "Calls", "In", "Out", "Cache", "Cost"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1);

        let rows: Vec<Row> = stats
            .models
            .iter()
            .map(|m| {
                let tier = get_tier(&m.model);
                let color = get_tier_color(&m.model);
                let cost = calculate_cost(m);

                let short_name = m.model
                    .replace("claude-", "")
                    .replace("-20", " '")
                    .chars()
                    .take(20)
                    .collect::<String>();

                Row::new(vec![
                    short_name,
                    tier.to_string(),
                    m.call_count.to_string(),
                    format_tokens(m.input_tokens),
                    format_tokens(m.output_tokens),
                    format_tokens(m.cache_read_tokens),
                    format_cost(cost),
                ])
                .style(Style::default().fg(color))
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Min(15),
                Constraint::Length(7),
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Models ")
                .title_style(Style::default().fg(Color::Magenta).bold())
                .padding(Padding::horizontal(1)),
        );

        frame.render_widget(table, area);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let data_age = self.last_data_refresh.elapsed().as_secs();

        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ←/→ ", Style::default().fg(Color::Yellow)),
            Span::styled("Period", Style::default().fg(Color::DarkGray)),
            Span::raw(" │ "),
            Span::styled("p ", Style::default().fg(Color::Yellow)),
            Span::styled("Plan", Style::default().fg(Color::DarkGray)),
            Span::raw(" │ "),
            Span::styled("r ", Style::default().fg(Color::Yellow)),
            Span::styled("Refresh", Style::default().fg(Color::DarkGray)),
            Span::raw(" │ "),
            Span::styled("q ", Style::default().fg(Color::Yellow)),
            Span::styled("Quit", Style::default().fg(Color::DarkGray)),
            Span::raw(" │ "),
            Span::styled(format!("Data: {}s ago", data_age), Style::default().fg(Color::DarkGray)),
        ]))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(footer, area);
    }
}
