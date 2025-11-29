"""JSONL parser for Claude usage data."""

import json
from pathlib import Path
from datetime import datetime, timedelta
from typing import Iterator, Optional

from .models import UsageEntry, ModelStats, PeriodStats


CLAUDE_DATA_DIR = Path.home() / ".claude" / "projects"


def find_jsonl_files(base_dir: Optional[Path] = None) -> list[Path]:
    """Find all JSONL files in Claude data directory."""
    base = base_dir or CLAUDE_DATA_DIR
    if not base.exists():
        return []
    return list(base.glob("**/*.jsonl"))


def parse_jsonl_file(filepath: Path) -> Iterator[UsageEntry]:
    """Parse entries from a single JSONL file."""
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    data = json.loads(line)
                    entry = UsageEntry.from_dict(data)
                    if entry:
                        yield entry
                except json.JSONDecodeError:
                    continue
    except (IOError, OSError):
        return


def parse_all_files(base_dir: Optional[Path] = None) -> list[UsageEntry]:
    """Parse all JSONL files and return sorted entries."""
    entries = []
    for filepath in find_jsonl_files(base_dir):
        entries.extend(parse_jsonl_file(filepath))
    return sorted(entries, key=lambda e: e.timestamp)


def filter_by_period(
    entries: list[UsageEntry],
    days: Optional[int] = None,
    start: Optional[datetime] = None,
    end: Optional[datetime] = None,
) -> list[UsageEntry]:
    """Filter entries by time period."""
    if days is not None:
        start = datetime.now().astimezone() - timedelta(days=days)
        end = datetime.now().astimezone()

    if start is None and end is None:
        return entries

    filtered = []
    for entry in entries:
        ts = entry.timestamp
        if start and ts < start:
            continue
        if end and ts > end:
            continue
        filtered.append(entry)

    return filtered


def aggregate_stats(entries: list[UsageEntry]) -> PeriodStats:
    """Aggregate entries into stats."""
    if not entries:
        now = datetime.now().astimezone()
        return PeriodStats(start=now, end=now)

    stats = PeriodStats(
        start=entries[0].timestamp,
        end=entries[-1].timestamp,
    )

    sessions = set()
    for entry in entries:
        sessions.add(entry.session_id)

        if entry.model not in stats.models:
            stats.models[entry.model] = ModelStats(model=entry.model)

        stats.models[entry.model].add_usage(entry.usage)

    stats.session_count = len(sessions)
    return stats
