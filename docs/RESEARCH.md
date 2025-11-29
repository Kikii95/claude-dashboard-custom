# Research & Sources — Claude Dashboard Custom

## 📍 Objectif

Ce document trace **toutes les sources d'information** utilisées pour implémenter le dashboard.

---

## 🗂️ Source de données : ~/.claude/projects/

### Découverte
```bash
ls ~/.claude/projects/
```

Les données de consommation Claude Code sont stockées dans des fichiers JSONL dans :
- `~/.claude/projects/<hash>/conversations/*.jsonl`

### Format d'une entrée JSONL
```json
{
  "type": "assistant",
  "timestamp": "2025-11-29T10:30:45.123Z",
  "sessionId": "abc123-...",
  "message": {
    "model": "claude-sonnet-4-20250514",
    "usage": {
      "input_tokens": 1500,
      "output_tokens": 800,
      "cache_creation_input_tokens": 0,
      "cache_read_input_tokens": 500
    }
  }
}
```

---

## ⏰ Système de Rate Limit : 5-Hour Blocks

### Source : claude-monitor

**Fichier analysé** :
`~/.local/share/uv/tools/claude-monitor/lib/python3.12/site-packages/claude_monitor/data/analyzer.py`

### Logique découverte

```python
# analyzer.py - Lignes 25-32
def __init__(self, session_duration_hours: int = 5):
    self.session_duration_hours = session_duration_hours
    self.session_duration = timedelta(hours=session_duration_hours)

# analyzer.py - Lignes 109-116
def _round_to_hour(self, timestamp: datetime) -> datetime:
    """Round timestamp to the nearest full hour in UTC."""
    return timestamp.replace(minute=0, second=0, microsecond=0)

# analyzer.py - Lignes 118-131
def _create_new_block(self, entry: UsageEntry) -> SessionBlock:
    """Create a new session block."""
    start_time = self._round_to_hour(entry.timestamp)  # <-- Début = heure arrondie
    end_time = start_time + self.session_duration       # <-- Fin = +5h = reset time!
```

### Règles extraites

| Règle | Valeur | Source |
|-------|--------|--------|
| Durée d'un block | **5 heures** | `session_duration_hours: int = 5` |
| Début du block | Première requête **arrondie à l'heure** | `_round_to_hour(entry.timestamp)` |
| Fin du block (= reset) | `start_time + 5h` | `end_time = start_time + self.session_duration` |
| Nouveau block si | Entry >= end_time OU gap >= 5h | `_should_create_new_block()` |

### Exemple concret

```
Première requête : 14:37 UTC
→ Block start    : 14:00 UTC (arrondi)
→ Block end      : 19:00 UTC (= reset time)
→ Countdown      : 4h 23m jusqu'au reset
```

---

## 💰 Pricing : $/Million tokens

### Source : claude-monitor

**Fichier analysé** :
`~/.local/share/uv/tools/claude-monitor/lib/python3.12/site-packages/claude_monitor/core/pricing.py`

### Pricing découvert (lignes 29-48)

```python
FALLBACK_PRICING: Dict[str, Dict[str, float]] = {
    "opus": {
        "input": 15.0,
        "output": 75.0,
        "cache_creation": 18.75,
        "cache_read": 1.5,
    },
    "sonnet": {
        "input": 3.0,
        "output": 15.0,
        "cache_creation": 3.75,
        "cache_read": 0.3,
    },
    "haiku": {
        "input": 0.25,
        "output": 1.25,
        "cache_creation": 0.3,
        "cache_read": 0.03,
    },
}
```

### Tableau récapitulatif ($/Million tokens)

| Tier | Input | Output | Cache Create | Cache Read |
|------|-------|--------|--------------|------------|
| **Opus** | $15.00 | $75.00 | $18.75 | $1.50 |
| **Sonnet** | $3.00 | $15.00 | $3.75 | $0.30 |
| **Haiku** | $0.25 | $1.25 | $0.30 | $0.03 |

---

## 📊 Plans & Limites

### Source : claude-monitor

**Fichier analysé** :
`~/.local/share/uv/tools/claude-monitor/lib/python3.12/site-packages/claude_monitor/core/plans.py`

### Limites découvertes (lignes 47-72)

```python
PLAN_LIMITS: Dict[PlanType, Dict[str, Any]] = {
    PlanType.PRO: {
        "token_limit": 19_000,
        "cost_limit": 18.0,
        "message_limit": 250,
    },
    PlanType.MAX5: {
        "token_limit": 88_000,
        "cost_limit": 35.0,
        "message_limit": 1_000,
    },
    PlanType.MAX20: {
        "token_limit": 220_000,
        "cost_limit": 140.0,
        "message_limit": 2_000,
    },
}
```

### Tableau récapitulatif (par fenêtre de 5h)

| Plan | Token Limit | Cost Limit | Message Limit |
|------|-------------|------------|---------------|
| **Pro** | 19K | $18 | 250 |
| **Max5** | 88K | $35 | 1,000 |
| **Max20** | 220K | $140 | 2,000 |

**Note** : Ces limites sont par fenêtre de 5 heures, pas par jour.
La limite atteinte EN PREMIER déclenche le rate limit.

---

## 🚨 Messages de Limite

### Source : Observation logs JSONL

Format typique dans les logs :
```
5-hour limit reached · resets 2am (Europe/Paris)
```

Le message de limite inclut :
- L'heure de reset en timezone locale
- Suggestion d'upgrade

**Fichier claude-monitor** : `analyzer.py` lignes 217-330 - Méthodes de détection des limites.

---

## 🏗️ SessionBlock Model

### Source : claude-monitor

**Fichier** : `~/.local/share/uv/tools/claude-monitor/lib/python3.12/site-packages/claude_monitor/core/models.py`

```python
@dataclass
class SessionBlock:
    """Aggregated session block representing a 5-hour period."""
    id: str
    start_time: datetime
    end_time: datetime
    entries: List[UsageEntry] = field(default_factory=list)
    token_counts: TokenCounts = field(default_factory=TokenCounts)
    is_active: bool = False
    is_gap: bool = False
    # ... etc
```

---

## 📁 Fichiers analysés

| Fichier | Chemin | Infos extraites |
|---------|--------|-----------------|
| analyzer.py | `claude_monitor/data/` | Logique 5h blocks, round_to_hour |
| models.py | `claude_monitor/core/` | Structures SessionBlock, UsageEntry |
| pricing.py | `claude_monitor/core/` | Pricing par modèle/token type |

---

## 🔗 Références externes

- [Anthropic Pricing](https://www.anthropic.com/pricing) - Tarifs officiels
- [Claude Code Documentation](https://docs.anthropic.com/en/docs/claude-code) - Doc officielle

---

## ✅ Implémentation dans claude-dashboard-custom

Basé sur ces recherches, implémenté dans :

| Notre fichier | Fonctionnalité |
|---------------|----------------|
| `src/parser.rs` | `create_blocks()`, `round_to_hour()`, `get_current_block_info()` |
| `src/calculator.rs` | `Pricing::OPUS/SONNET/HAIKU`, `calculate_cost()` |
| `src/models.rs` | `SessionBlock`, `CurrentBlockInfo`, `PlanLimits` |
| `src/ui/app.rs` | `draw_current_block()` avec reset time |

---

**Document généré lors du développement v0.4.0**
