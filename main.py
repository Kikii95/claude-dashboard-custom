#!/usr/bin/env python3
"""Claude Dashboard - Track your Claude Code usage."""

import argparse
import sys
from pathlib import Path

from claude_dashboard.parser import parse_all_files, filter_by_period, aggregate_stats
from claude_dashboard.dashboard import render_dashboard, render_compact, console


def main():
    parser = argparse.ArgumentParser(
        description="Dashboard for Claude Code usage tracking",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                    # Show dashboard for last 30 days
  %(prog)s -d 7               # Last 7 days
  %(prog)s -d 1               # Today only
  %(prog)s --plan max5        # Use Max5 plan limits
  %(prog)s --compact          # Single line output
        """
    )

    parser.add_argument(
        "-d", "--days",
        type=int,
        default=30,
        help="Number of days to analyze (default: 30)"
    )

    parser.add_argument(
        "-p", "--plan",
        choices=["pro", "max5", "max20"],
        default="pro",
        help="Plan to compare against (default: pro)"
    )

    parser.add_argument(
        "-c", "--compact",
        action="store_true",
        help="Compact single-line output"
    )

    parser.add_argument(
        "--data-dir",
        type=Path,
        help="Custom data directory (default: ~/.claude/projects)"
    )

    parser.add_argument(
        "-v", "--version",
        action="version",
        version="%(prog)s 0.1.0"
    )

    args = parser.parse_args()

    # Parse data
    try:
        entries = parse_all_files(args.data_dir)
    except Exception as e:
        console.print(f"[red]Error reading data: {e}[/red]")
        sys.exit(1)

    if not entries:
        console.print("[yellow]No usage data found in ~/.claude/projects/[/yellow]")
        sys.exit(0)

    # Filter by period
    filtered = filter_by_period(entries, days=args.days)

    if not filtered:
        console.print(f"[yellow]No data found for last {args.days} days[/yellow]")
        sys.exit(0)

    # Aggregate stats
    stats = aggregate_stats(filtered)

    # Render
    if args.compact:
        render_compact(stats, args.plan)
    else:
        render_dashboard(stats, args.plan)


if __name__ == "__main__":
    main()
