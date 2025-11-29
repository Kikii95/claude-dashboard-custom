"""Data models for Claude usage tracking."""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional


@dataclass
class TokenUsage:
    """Token usage from a single API call."""
    input_tokens: int = 0
    output_tokens: int = 0
    cache_creation_input_tokens: int = 0
    cache_read_input_tokens: int = 0

    @property
    def total_tokens(self) -> int:
        return (
            self.input_tokens +
            self.output_tokens +
            self.cache_creation_input_tokens +
            self.cache_read_input_tokens
        )


@dataclass
class UsageEntry:
    """Single usage entry from JSONL."""
    timestamp: datetime
    session_id: str
    model: str
    usage: TokenUsage

    @classmethod
    def from_dict(cls, data: dict) -> Optional["UsageEntry"]:
        """Parse from JSONL dict."""
        try:
            message = data.get("message", {})
            usage_data = message.get("usage", {})

            if not usage_data:
                return None

            return cls(
                timestamp=datetime.fromisoformat(
                    data["timestamp"].replace("Z", "+00:00")
                ),
                session_id=data.get("sessionId", "unknown"),
                model=message.get("model", "unknown"),
                usage=TokenUsage(
                    input_tokens=usage_data.get("input_tokens", 0),
                    output_tokens=usage_data.get("output_tokens", 0),
                    cache_creation_input_tokens=usage_data.get("cache_creation_input_tokens", 0),
                    cache_read_input_tokens=usage_data.get("cache_read_input_tokens", 0),
                )
            )
        except (KeyError, ValueError):
            return None


@dataclass
class ModelStats:
    """Aggregated stats for a model."""
    model: str
    total_input: int = 0
    total_output: int = 0
    total_cache_create: int = 0
    total_cache_read: int = 0
    call_count: int = 0

    def add_usage(self, usage: TokenUsage) -> None:
        self.total_input += usage.input_tokens
        self.total_output += usage.output_tokens
        self.total_cache_create += usage.cache_creation_input_tokens
        self.total_cache_read += usage.cache_read_input_tokens
        self.call_count += 1

    @property
    def total_tokens(self) -> int:
        return (
            self.total_input +
            self.total_output +
            self.total_cache_create +
            self.total_cache_read
        )


@dataclass
class PeriodStats:
    """Stats for a time period."""
    start: datetime
    end: datetime
    models: dict[str, ModelStats] = field(default_factory=dict)
    session_count: int = 0

    @property
    def total_tokens(self) -> int:
        return sum(m.total_tokens for m in self.models.values())

    @property
    def total_calls(self) -> int:
        return sum(m.call_count for m in self.models.values())
