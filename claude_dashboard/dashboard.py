"""Rich TUI dashboard for Claude usage stats."""

from datetime import datetime

from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.layout import Layout
from rich.text import Text
from rich import box

from .models import PeriodStats
from .calculator import (
    calculate_model_cost,
    total_cost,
    estimate_plan_usage,
    get_model_tier,
)


console = Console()


def format_tokens(count: int) -> str:
    """Format token count with K/M suffix."""
    if count >= 1_000_000:
        return f"{count / 1_000_000:.2f}M"
    elif count >= 1_000:
        return f"{count / 1_000:.1f}K"
    return str(count)


def format_cost(cost: float) -> str:
    """Format cost in dollars."""
    return f"${cost:.4f}"


def create_header() -> Panel:
    """Create header panel."""
    title = Text()
    title.append("⚡ ", style="yellow")
    title.append("CLAUDE DASHBOARD", style="bold cyan")
    title.append(" ⚡", style="yellow")

    subtitle = Text()
    subtitle.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M')}", style="dim")

    content = Text.assemble(title, "\n", subtitle)
    return Panel(content, box=box.DOUBLE, style="cyan")


def create_summary_panel(stats: PeriodStats) -> Panel:
    """Create summary stats panel."""
    cost = total_cost(stats)

    grid = Table.grid(padding=(0, 2))
    grid.add_column(justify="right", style="dim")
    grid.add_column(justify="left", style="bold")

    grid.add_row("📊 Total Tokens:", format_tokens(stats.total_tokens))
    grid.add_row("💰 Total Cost:", format_cost(cost))
    grid.add_row("📞 API Calls:", f"{stats.total_calls:,}")
    grid.add_row("🔗 Sessions:", f"{stats.session_count:,}")
    grid.add_row("📅 Period:", f"{stats.start.strftime('%m/%d')} - {stats.end.strftime('%m/%d')}")

    return Panel(grid, title="[bold]📈 Summary[/bold]", border_style="green")


def create_models_table(stats: PeriodStats) -> Table:
    """Create models breakdown table."""
    table = Table(
        title="🤖 Usage by Model",
        box=box.ROUNDED,
        show_header=True,
        header_style="bold magenta",
    )

    table.add_column("Model", style="cyan", no_wrap=True)
    table.add_column("Tier", style="yellow")
    table.add_column("Calls", justify="right")
    table.add_column("Input", justify="right", style="green")
    table.add_column("Output", justify="right", style="blue")
    table.add_column("Cache R", justify="right", style="dim")
    table.add_column("Cost", justify="right", style="bold yellow")

    # Sort by cost descending
    sorted_models = sorted(
        stats.models.items(),
        key=lambda x: calculate_model_cost(x[1]),
        reverse=True
    )

    for model, model_stats in sorted_models:
        cost = calculate_model_cost(model_stats)
        tier = get_model_tier(model)

        # Shorten model name
        short_name = model.replace("claude-", "").replace("-20", " ")[:25]

        table.add_row(
            short_name,
            tier,
            f"{model_stats.call_count:,}",
            format_tokens(model_stats.total_input),
            format_tokens(model_stats.total_output),
            format_tokens(model_stats.total_cache_read),
            format_cost(cost),
        )

    return table


def create_plan_panel(stats: PeriodStats, plan: str = "pro") -> Panel:
    """Create plan usage panel with progress bars."""
    usage = estimate_plan_usage(stats, plan)
    if not usage:
        return Panel("Unknown plan", title="Plan Usage")

    lines = []

    # Cost progress
    cost_pct = min(usage["cost_percent"], 100)
    cost_bar = "█" * int(cost_pct / 5) + "░" * (20 - int(cost_pct / 5))
    cost_color = "green" if cost_pct < 70 else "yellow" if cost_pct < 90 else "red"
    lines.append(f"💰 Cost:  [{cost_color}]{cost_bar}[/] {cost_pct:.1f}%")
    lines.append(f"         {format_cost(usage['cost_used'])} / {format_cost(usage['cost_limit'])}")

    lines.append("")

    # Calls progress
    calls_pct = min(usage["calls_percent"], 100)
    calls_bar = "█" * int(calls_pct / 5) + "░" * (20 - int(calls_pct / 5))
    calls_color = "green" if calls_pct < 70 else "yellow" if calls_pct < 90 else "red"
    lines.append(f"📞 Calls: [{calls_color}]{calls_bar}[/] {calls_pct:.1f}%")
    lines.append(f"         {usage['calls_used']:,} / {usage['calls_limit']:,}")

    content = "\n".join(lines)
    return Panel(content, title=f"[bold]📋 Plan: {plan.upper()}[/bold]", border_style="blue")


def render_dashboard(stats: PeriodStats, plan: str = "pro") -> None:
    """Render the full dashboard."""
    console.clear()

    # Header
    console.print(create_header())
    console.print()

    # Summary and Plan side by side
    layout = Layout()
    layout.split_row(
        Layout(create_summary_panel(stats), name="summary"),
        Layout(create_plan_panel(stats, plan), name="plan"),
    )
    console.print(layout)
    console.print()

    # Models table
    console.print(create_models_table(stats))
    console.print()

    # Footer
    console.print(
        "[dim]Press Ctrl+C to exit | "
        "Data from ~/.claude/projects/*.jsonl[/dim]",
        justify="center"
    )


def render_compact(stats: PeriodStats, plan: str = "pro") -> None:
    """Render compact single-line output."""
    cost = total_cost(stats)
    usage = estimate_plan_usage(stats, plan)

    if usage:
        console.print(
            f"[cyan]Claude[/cyan] | "
            f"[yellow]${cost:.2f}[/yellow] ({usage['cost_percent']:.0f}%) | "
            f"[green]{format_tokens(stats.total_tokens)}[/green] tokens | "
            f"[blue]{stats.total_calls}[/blue] calls"
        )
    else:
        console.print(
            f"[cyan]Claude[/cyan] | "
            f"[yellow]${cost:.2f}[/yellow] | "
            f"[green]{format_tokens(stats.total_tokens)}[/green] tokens"
        )
