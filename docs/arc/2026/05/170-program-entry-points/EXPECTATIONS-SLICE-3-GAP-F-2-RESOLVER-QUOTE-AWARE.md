# Arc 170 slice 3 Gap F-2 EXPECTATIONS (sonnet scorecard)

**One spawn.** Resolver quote-awareness fix. Most design-heavy of Phase 2a gap slices (quasiquote-unquote semantics).

## Independent prediction

**Runtime band:** 45-90 min sonnet.

**Hard cap:** 180 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Resolver `:wat::core::forms` arm: don't recurse into children | grep + read |
| B | Resolver `:wat::core::quote` arm: don't recurse into argument | grep + read |
| C | Resolver `:wat::core::quasiquote` arm: recurse with unquote-aware descent | grep + read |
| D | 3+ probes pass: forms / quote / quasiquote+unquote | cargo test |
| E | Workspace at baseline + new probes / 0 failed | full test |
| F | Existing quote-using code unchanged behavior | full test |

**6 rows.** All must PASS.

## Implementation approach

1. **Audit current behavior** (10 min): grep `check_form` / `resolve_references`; map current quote-handling
2. **Probes** (15 min): 3+ probes confirming failure baseline
3. **Extend resolver** (20-30 min): three arms (forms, quote, quasiquote+unquote)
4. **Verify** (15-30 min): probes + workspace + regression check

## What sonnet produces

- `src/resolve.rs` (or wherever `check_form` lives) modified
- 1 new probe test file
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-F-2-RESOLVER-QUOTE-AWARE.md` with:
  - 6-row scorecard
  - Current resolver behavior audit
  - Nested quasiquote disposition
  - Other resolver call sites (sibling walkers needing same arms?)
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify quote-family form REGISTRATION (special_forms.rs unchanged)
- Modify macro EXPANSION logic
- Add new quote-family forms
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Extend to Gap F-1 / F-3 / G scope
- Use --no-verify or skip hooks
- Break existing quote-using code

## Verification commands

```bash
# New F-2 probes
cargo test --release --test probe_resolver_quote_awareness 2>&1 | tail -5

# Regression: all prior substrate probes
cargo test --release --test probe_do_splice_def 2>&1 | tail -3
cargo test --release --test probe_let_splice_def 2>&1 | tail -3
cargo test --release --test probe_do_splice_define 2>&1 | tail -3
cargo test --release --test probe_let_splice_define 2>&1 | tail -3
# Plus F-1 + F-3 probes
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post-F-1+F-3): 2209 + N + 3 / 0 failed
- Post-F-2: 2209 + N + 3 + 3 / 0 failed

## Honest delta categories (anticipated)

1. **Nested quasiquote disposition** — current behavior + whether F-2 addresses it
2. **Unquote-splicing edge cases** — `~@list` semantics
3. **Other resolver call sites** — sibling walkers needing same arms
4. **Existing quote-using code impact** — any prior reliance on walking-into-quote behavior
5. **Resolver-vs-macro-expansion ordering surprises**
