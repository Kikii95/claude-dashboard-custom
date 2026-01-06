// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use claude_dashboard_lib::{
    aggregate, filter_this_month, filter_this_week, filter_today,
    get_current_block_info, get_model_distribution, parse_all,
    DashboardData, PlanLimits, PLANS,
};

/// Get all dashboard data for display
#[tauri::command]
fn get_dashboard_data(plan_index: usize) -> Result<DashboardData, String> {
    let entries = parse_all().map_err(|e| e.to_string())?;

    let plan_index = plan_index.min(PLANS.len().saturating_sub(1));
    let selected_plan = PLANS.get(plan_index).cloned().unwrap_or_else(|| PlanLimits {
        name: "Unknown".into(),
        token_limit: 0,
        cost_limit: 0.0,
        message_limit: 0,
    });

    let today_entries = filter_today(&entries);
    let week_entries = filter_this_week(&entries);
    let month_entries = filter_this_month(&entries);

    let current_block = get_current_block_info(&entries, &selected_plan);
    let today = aggregate(&today_entries, "Today");
    let week = aggregate(&week_entries, "This Week");
    let month = aggregate(&month_entries, "This Month");
    let model_distribution = get_model_distribution(&entries);

    // Generate warnings based on usage
    let mut warnings = Vec::new();
    if current_block.cost_percent >= 90.0 {
        warnings.push("⚠️ Cost limit nearly exhausted (90%+)".to_string());
    }
    if current_block.tokens_percent >= 90.0 {
        warnings.push("⚠️ Token limit nearly exhausted (90%+)".to_string());
    }
    if current_block.messages_percent >= 90.0 {
        warnings.push("⚠️ Message limit nearly exhausted (90%+)".to_string());
    }
    if current_block.cost_percent >= 100.0 || current_block.tokens_percent >= 100.0 {
        warnings.push("🚨 RATE LIMITED - Wait for reset!".to_string());
    }

    Ok(DashboardData {
        current_block,
        today,
        week,
        month,
        selected_plan,
        model_distribution,
        warnings,
    })
}

/// Get available plans for selection
#[tauri::command]
fn get_available_plans() -> Vec<PlanLimits> {
    PLANS.clone()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_dashboard_data, get_available_plans])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
