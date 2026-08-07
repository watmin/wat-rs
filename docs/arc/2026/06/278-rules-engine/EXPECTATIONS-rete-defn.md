# EXPECTATIONS — #88 v2: the check moves to registration, the refusal becomes a value

Written **before** the strike. Scored by the orchestrator's own re-run, never the rider's report —
v1's report was wrong in two places that only an independent run surfaced.

## Baseline

```
HEAD cb98db6a (docs+probe only)   floor at HEAD 4376/0
tree: v1's substrate work + 3 corrections + 14 re-headings, UNCOMMITTED, floor RED by construction
```

The red is the starting condition, not a regression. v1's own measured red was **17 failed**.

## The scorecard

| # | what | the command | expected |
|---|---|---|---|
| 1 | **the marker survives registration** | the 14 re-headed files load AND their rules compile | green — this is the whole strike |
| 2 | **one door** | `grep -c apply_rete_defn_contracts src/` | exactly one call site; `freeze/env.rs`'s step 6.975 is GONE (STOP-3) |
| 3 | **the live-session path is covered** | a rete-defn registered via `runtime.rs:24475`, not only at boot | checked + stamped there too (STOP-5) |
| 4 | **`pure?` is still honest** | `probe_arc278_6a_purity` | green — an ordinary pure fn answers `pure? = true` (STOP-2) |
| 5 | **the membrane still bites** | the acceptance gate, mutated both ways | RED with a non-rete body, GREEN when rete-clean — proven both directions |
| 6 | **the error still names the HELPER** | the gate's diagnostic | carries `:probe::declared`, located at the declaration |
| 7 | **the refusal is a VALUE** | read the type | a caller can `match` it; shaped like `SiftRulesResponse` — one good, N named bad, each with located fields |
| 8 | **no new raise at the boundary** | `git diff` | the refusal path adds no `raise!`/`assertion-failed!` |
| 9 | self-recursion still admitted | a law-A-clean self-recursive rete-defn | loads and runs (it printed `0` before; must still) |
| 10 | group order still irrelevant | `where-nesting`'s `c1`/`c2` | passes regardless of hash iteration order |
| 11 | the four walks are REUSED | `grep -c 'fn is_pure_expr\|fn is_total_expr' src/rete/purity.rs` | unchanged (STOP-1) |
| 12 | codemod idempotent | re-run it | 0 changed |
| 13 | stdlib loads | `:wat::deporder::verify-stdlib` | prints `[]` |
| 14 | **the floor** | `scripts/floor.sh` → the **Summary line** | ≥ 4376 passed, **0 failed** |
| 15 | clippy | `cargo clippy --release --all-targets` | 0 warnings |

Rows **1, 3, 4, 7** are load-bearing. Row 1 is the strike. Row 3 is the one that makes it survive
contact with the deployment model. Row 4 is the one that went wrong invisibly last time — it broke
nine tests in files nobody was looking at.

## Runtime prediction

**40–70 min.** The move itself is small; deriving the declared-name set at the new call site and
threading it is the real work, and row 3 may surface a signature problem. Overrun most likely means
STOP-5 fired.

## Trap doors, named in advance

1. **Two call sites.** The tempting shape is "keep the boot check AND add registration." That is
   STOP-3 and it is two implementations of one law.
2. **The session path is harder than the boot path.** `runtime.rs:24475` builds a `session_sym`; the
   declared-name set may not be in scope there. That is a real signature question, not a reason to
   skip it — skipping it rebuilds the exact defect being fixed.
3. **Re-widening the membrane.** Denying all four axes is simpler to write and silently wrong.
4. **A green build over a red corpus.** The bake does not run the sweep. Row 14 is the arbiter.
5. **Trusting a `head`-truncated run.** v1's orchestrator read two lines of a 15-line output and
   called an axis green when it exited 2. Read whole outputs; read the Summary line.
6. **A gate that cannot go red.** Row 5 must be shown BOTH ways by mutation.

## Mode B

Any of: two call sites survive; the session path is silently skipped; the membrane re-widens past
law A; a new raise appears on the refusal path; or the floor is called green from a piped exit code
rather than the Summary line.
