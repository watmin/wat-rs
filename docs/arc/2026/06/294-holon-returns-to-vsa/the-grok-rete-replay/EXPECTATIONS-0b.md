# EXPECTATIONS 0b — written BEFORE the strike

Base: `replay/grok-rete` @ `0d51a1305`. Gate: 13 fixtures · 83 ledger · 0 runes · 96 stems.

## Batch 1 — the 17 chain codemods

| # | what | command | expected |
|---|---|---|---|
| B1.1 | 17 fixtures land | `ls wat-scripts/fixes/replay/` | 30 dirs (13 + 17), including all 17 named in the brief |
| B1.2 | the ledger shrinks by exactly those 17 names | `FROZEN_LEDGER` entries | 66 left, none of the 17 among them |
| B1.3 | SCOPE on the 28 chain codemods + A + B | `grep -c '^;; SCOPE: ' wat-scripts/fixes/<stem>.wat` | 1 each |
| B1.4 | tooling scopes are not `corpus` | the SCOPE lines of `fmt-head-fqdn-to-clojure`, `break-kind-string-to-enum`, `node-kind-string-to-enum`, `edits-carry-the-old-text` | explicit paths or globs, each matching ≥1 tracked file |
| B1.5 | oracle provenance | the SCORE cites commit + file (or "spec") per fixture. The orchestrator re-checks every changed line against the cited commit, whitespace-normalized | 17/17 cited; every changed line found, or declared spec |
| B1.6 | coverage | the SCORE maps every documented case / defrule to a fixture line, and names each near-miss | complete; ≥1 near-miss per fixture |
| B1.7 | the gate | `cargo nextest run --release --test cli -E 'test(every_recorded_migration_replays)'` | all pass |
| B1.8 | floor / clippy | `scripts/floor.sh`; `cargo clippy --release --all-targets` | 0 failed · 0 lines |
| B1.9 | committed, not pushed | `git log origin/replay/grok-rete..HEAD` | ≥1 commit; no push |

## Batch 2 — the other 66, the repairs, the exemptions, the final gate

| # | what | command | expected |
|---|---|---|---|
| B2.1 | the ledger is gone | `grep -c FROZEN_LEDGER tests/cli/every_recorded_migration_replays.rs` | 0 |
| B2.2 | 96 = fixtures + runes | the gate's coverage fn | fixtures + runes == 96; no stem in both |
| B2.3 | runes are honest | each `rune:replay(unreadable-preimage)` reason | the reader's VERBATIM error, and the orchestrator reproduces it by running that codemod on its pre-image |
| B2.4 | rot repaired | `to-faithful-clojure-net` / `-rete` | fixtures pass; no user fn left in any `:wat::rete::where`; root cause in the SCORE |
| B2.5 | every other repair explained | the SCORE's repair list | per repair: the symptom, the root cause, the change, and the fixture that proves it |
| B2.6 | SCOPE on all 96 | `grep -L '^;; SCOPE: ' wat-scripts/fixes/*.wat` | empty |
| B2.7 | the new gate arms fail | the brief's (e)–(h) | each RED, naming the offender; verbatim text in the SCORE |
| B2.8 | provenance, coverage, near-misses | as in B1.5 and B1.6, over all 66 | complete |
| B2.9 | floor / clippy | as in B1.8 | 0 failed · 0 lines |

**Runtime prediction:** batch 1 takes 3–6 h. Batch 2 takes 8–15 h, most of it choosing
history-derived forms, plus the repairs.

**Trap doors, named in advance:**
- **The oracle temptation.** B1.5 and B2.8 exist to catch it.
- **A landing commit that mixed hand edits into the codemod's hunks.** Choose forms changed ONLY by
  the codemod.
- **`positional-ctor-to-map`** costs ~10 s per run, and the gate runs it twice.
- **`one-param-spec`** reads two path vectors, so its fixture needs a `stdin` template.
- **An exemption granted by assumption.** Every rune must reproduce (B2.3).
- **A rotted codemod "repaired" by changing what it does.** That is STOP-1.
