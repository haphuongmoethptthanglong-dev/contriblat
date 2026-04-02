# ContribAI v5.4.0

## 🌙 Dream Memory Consolidation

New `dream` system that consolidates scattered memory entries into durable, well-organized repo profiles.

- **3-gate trigger**: Runs automatically when 24h have passed + 5 sessions completed + no concurrent dream
- **Repo profiling**: Aggregates PR outcomes (merged/closed), feedback patterns, and review times into personality profiles per repo
- **Smarter targeting**: Dream profiles inform which contribution types to prioritize for each repo
- **Memory pruning**: Expired working memory entries are automatically cleaned up
- **CLI**: `contribai dream` (check status) / `contribai dream --force` (run now)

## ⚡ Risk Classification Engine

Every generated contribution is now classified by risk level before submission:

| Risk Level | Examples | Auto-Submit |
|:-----------|:---------|:------------|
| **LOW** | Docs, typos, formatting, imports | ✅ Yes |
| **MEDIUM** | Small bug fixes, security patches | ✅ With review note |
| **HIGH** | Multi-file refactors, behavior changes | ❌ Requires `--approve` |

- Configurable via `pipeline.risk_tolerance` in `config.yaml` (low/medium/high)
- Default tolerance: `medium` — docs + small fixes auto-submit, large changes held

## 🔧 Other Changes

- Interactive menu now includes 💤 Dream entry (23 total commands)
- `dream_meta` table added to memory.db for consolidation tracking
- Session counter tracks pipeline runs for dream gate logic
- `get_leaderboard()` method for repo merge rate rankings
- 18 new tests (353+ total)

## Installation

```bash
cargo install --path crates/contribai-rs
```
