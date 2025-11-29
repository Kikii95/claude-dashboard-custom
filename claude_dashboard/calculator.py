"""Cost calculator for Claude API usage."""

from dataclasses import dataclass
from typing import Optional

from .models import ModelStats, PeriodStats


@dataclass
class ModelPricing:
    """Pricing per million tokens for a model."""
    input: float
    output: float
    cache_create: float
    cache_read: float


# Pricing per million tokens (as of late 2024)
PRICING: dict[str, ModelPricing] = {
    # Opus
    "claude-opus-4-5-20251101": ModelPricing(15.0, 75.0, 18.75, 1.50),
    "claude-3-opus-20240229": ModelPricing(15.0, 75.0, 18.75, 1.50),
    # Sonnet
    "claude-sonnet-4-5-20250929": ModelPricing(3.0, 15.0, 3.75, 0.30),
    "claude-3-5-sonnet-20241022": ModelPricing(3.0, 15.0, 3.75, 0.30),
    "claude-3-5-sonnet-20240620": ModelPricing(3.0, 15.0, 3.75, 0.30),
    "claude-3-sonnet-20240229": ModelPricing(3.0, 15.0, 3.75, 0.30),
    # Haiku
    "claude-3-5-haiku-20241022": ModelPricing(0.25, 1.25, 0.30, 0.03),
    "claude-3-haiku-20240307": ModelPricing(0.25, 1.25, 0.30, 0.03),
}

# Plan limits
PLAN_LIMITS = {
    "pro": {"tokens": 19_000, "cost": 18.0, "messages": 250},
    "max5": {"tokens": 88_000, "cost": 35.0, "messages": 1000},
    "max20": {"tokens": 220_000, "cost": 140.0, "messages": 2000},
}


def get_pricing(model: str) -> ModelPricing:
    """Get pricing for a model, with fallback to Sonnet pricing."""
    if model in PRICING:
        return PRICING[model]

    # Fallback based on model name
    model_lower = model.lower()
    if "opus" in model_lower:
        return ModelPricing(15.0, 75.0, 18.75, 1.50)
    elif "haiku" in model_lower:
        return ModelPricing(0.25, 1.25, 0.30, 0.03)
    else:
        # Default to Sonnet
        return ModelPricing(3.0, 15.0, 3.75, 0.30)


def calculate_model_cost(stats: ModelStats) -> float:
    """Calculate cost for a model's usage."""
    pricing = get_pricing(stats.model)

    cost = 0.0
    cost += (stats.total_input / 1_000_000) * pricing.input
    cost += (stats.total_output / 1_000_000) * pricing.output
    cost += (stats.total_cache_create / 1_000_000) * pricing.cache_create
    cost += (stats.total_cache_read / 1_000_000) * pricing.cache_read

    return cost


def calculate_period_cost(stats: PeriodStats) -> dict[str, float]:
    """Calculate costs for all models in a period."""
    costs = {}
    for model, model_stats in stats.models.items():
        costs[model] = calculate_model_cost(model_stats)
    return costs


def total_cost(stats: PeriodStats) -> float:
    """Calculate total cost for a period."""
    return sum(calculate_period_cost(stats).values())


def get_model_tier(model: str) -> str:
    """Get human-readable tier name for a model."""
    model_lower = model.lower()
    if "opus" in model_lower:
        return "Opus"
    elif "haiku" in model_lower:
        return "Haiku"
    else:
        return "Sonnet"


def estimate_plan_usage(stats: PeriodStats, plan: str = "pro") -> Optional[dict]:
    """Estimate usage against a plan limit."""
    if plan not in PLAN_LIMITS:
        return None

    limits = PLAN_LIMITS[plan]
    cost = total_cost(stats)

    return {
        "plan": plan,
        "cost_used": cost,
        "cost_limit": limits["cost"],
        "cost_percent": (cost / limits["cost"]) * 100 if limits["cost"] > 0 else 0,
        "calls_used": stats.total_calls,
        "calls_limit": limits["messages"],
        "calls_percent": (stats.total_calls / limits["messages"]) * 100 if limits["messages"] > 0 else 0,
    }
