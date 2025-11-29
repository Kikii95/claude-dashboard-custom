use std::time::{Duration, Instant};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Padding, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::calculator::{calculate_cost, format_cost, format_tokens, get_tier, get_tier_color};
use crate::models::{Entry, PeriodStats, PlanLimits, PLANS};
use crate::parser::{aggregate, filter_this_month, filter_this_week, filter_today, parse_all};

const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub struct App {
    pub entries: Vec<Entry>,
    pub today: PeriodStats,
    pub week: PeriodStats,
    pub month: PeriodStats,
    pub all_time: PeriodStats,
    pub selected_plan: usize,
    pub selected_period: usize,
    pub last_refresh: Instant,
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
            selected_plan: 0,
            selected_period: 0,
            last_refresh: Instant::now(),
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
        }
        self.last_refresh = Instant::now();
    }

    pub fn maybe_refresh(&mut self) {
        if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
            self.refresh();
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
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Main layout: header, content, footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Content
                Constraint::Length(1),  // Footer
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
        // Split into left (stats) and right (models)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        self.draw_stats_panel(frame, chunks[0]);
        self.draw_models_panel(frame, chunks[1]);
    }

    fn draw_stats_panel(&self, frame: &mut Frame, area: Rect) {
        let stats = self.current_stats();
        let plan = self.current_plan();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Summary
                Constraint::Length(7),  // Plan progress
                Constraint::Min(5),     // Cost breakdown
            ])
            .split(area);

        // Summary block
        let summary_text = vec![
            Line::from(vec![
                Span::styled("  💰 Cost:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_cost(stats.total_cost), Style::default().fg(Color::Yellow).bold()),
            ]),
            Line::from(vec![
                Span::styled("  📊 Tokens:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format_tokens(stats.total_tokens), Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(vec![
                Span::styled("  📞 Calls:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", stats.total_calls), Style::default().fg(Color::Blue).bold()),
            ]),
            Line::from(vec![
                Span::styled("  🔗 Sessions:", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {}", stats.session_count), Style::default().fg(Color::Magenta).bold()),
            ]),
        ];

        let summary = Paragraph::new(summary_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(format!(" {} ", stats.period_label))
                    .title_style(Style::default().fg(Color::Green).bold())
                    .padding(Padding::vertical(1)),
            );

        frame.render_widget(summary, chunks[0]);

        // Plan progress
        let cost_pct = ((stats.total_cost / plan.cost_limit) * 100.0).min(100.0) as u16;
        let calls_pct = ((stats.total_calls as f64 / plan.message_limit as f64) * 100.0).min(100.0) as u16;

        let cost_color = if cost_pct < 70 { Color::Green } else if cost_pct < 90 { Color::Yellow } else { Color::Red };
        let calls_color = if calls_pct < 70 { Color::Green } else if calls_pct < 90 { Color::Yellow } else { Color::Red };

        let plan_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(chunks[1]);

        let cost_gauge = Gauge::default()
            .block(Block::default().title(format!("Cost: {} / {}", format_cost(stats.total_cost), format_cost(plan.cost_limit))))
            .gauge_style(Style::default().fg(cost_color))
            .percent(cost_pct)
            .label(format!("{}%", cost_pct));

        let calls_gauge = Gauge::default()
            .block(Block::default().title(format!("Calls: {} / {}", stats.total_calls, plan.message_limit)))
            .gauge_style(Style::default().fg(calls_color))
            .percent(calls_pct)
            .label(format!("{}%", calls_pct));

        let plan_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(format!(" Plan: {} [p] ", plan.name))
            .title_style(Style::default().fg(Color::Blue).bold());

        frame.render_widget(plan_block, chunks[1]);
        frame.render_widget(cost_gauge, plan_chunks[0]);
        frame.render_widget(calls_gauge, plan_chunks[1]);

        // Cost by tier
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
                    Span::styled(format!("  {} ", tier), Style::default().fg(*color).bold()),
                    Span::styled(format_cost(*cost), Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let tier_block = Paragraph::new(tier_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" By Tier ")
                    .padding(Padding::vertical(1)),
            );

        frame.render_widget(tier_block, chunks[2]);
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

                // Shorten model name
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
        let elapsed = self.last_refresh.elapsed().as_secs();
        let next_refresh = if REFRESH_INTERVAL.as_secs() > elapsed {
            REFRESH_INTERVAL.as_secs() - elapsed
        } else {
            0
        };

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
            Span::styled(format!("⟳ {}s", next_refresh), Style::default().fg(Color::DarkGray)),
        ]))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(footer, area);
    }
}
