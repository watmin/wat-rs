# SCORE — Stone 254.0: `make-channel`, the one canonical channel constructor

Scored against an independent orchestrator re-run (weigh, don't rubber-stamp).
**PASS — the channel-construction surface is collapsed to one verb; clean sheet
across the whole tree.**

## Scorecard

| # | check | result |
|---|---|---|
| 1 | probe A/B/C/D/E un-ignored | **PASS** — 3/3 (make-channel checks; unbounded + bounded(N) + phantom queues all rejected) |
| 2 | lib baseline | **PASS** — 938/0/1 (was 940; the −2 is the kill — the non-negative-capacity + non-keyword-arg tests tested validation that no longer exists) |
| 3 | full test build (all targets) | **PASS** — `cargo build --release --tests` exit 0 |
| 4 | stdlib runtime (stream.wat) | **PASS** — `wat_stream` 22/0; the prelude loads make-channel on every freeze |
| 5 | migrated rust tests | **PASS** — `types/typealias` 9/0 |
| 6 | clippy | **PASS** — no channel/queue warnings |
| 7 | three-nil grep (whole tree) | **PASS** — zero LIVE retired-verb call sites (only the LEGACY redirect-diagnostic registry + the disconfirming probe fixtures + one frozen `docs/` archive snapshot) |
| 8 | LEGACY registry semantics | **PASS** — a reject-diagnostic ("use make-channel"), NOT a working alias; HARD-CUT-compatible |

## What shipped

Kernel: `eval_make_bounded_queue`+`eval_make_unbounded_queue` → `eval_make_channel`
(arity-1, always `bounded(1)`); `infer_make_queue` → `infer_make_channel`; the four
condemned verb arms (2 channel + 2 phantom queue) collapsed to one; `typed_channel::
unbounded()` deleted (`bounded()` kept). Migrated every call site across stdlib wat,
the wat corpus, the rust tests, AND the `crates/` consumers.

## Score caught (the value of weighing)

1. **Cascade incompleteness — sonnet's brief grep-scope missed `crates/`.** The
   wat-corpus suite went RED (19 HologramCacheService failures) on a live
   `make-bounded-channel` in `crates/wat-holon-lru` — outside the briefed
   `src/ wat/ wat-tests/ tests/` scope. Whole-tree grep found callers in
   wat-lru, wat-holon-lru, wat-telemetry, wat-telemetry-sqlite. Corrected.
2. **My OWN grounding error (marked, not buried).** I told the builder
   "`make-bounded-channel N` with N≠1 has zero call sites" — that grep was scoped
   to 4 dirs and **missed `crates/`**, where wat-telemetry test fixtures use
   `bounded(16)` and `bounded(4)`. The builder decided "drop bounded" partly on
   that wrong claim. Re-grounded against the whole tree; builder reconfirmed
   **depth-1 is universal — the N≠1 callers are in violation and were corrected**
   (telemetry stub channels → depth-1; producer/consumer run on different threads
   so lock-step holds, the 16/4 was slack).
3. **A Rust-level miss the wat-verb grep couldn't see.** `probe_channel_primitive.rs`
   imported `wat::typed_channel::unbounded` (deleted) — migrated to the surviving
   `bounded(1)` depth-1 primitive with honest names.

## REVEALED, not caused — pre-existing rot (SEPARATE, banked)

Collapsing the channel un-masked failures that the make-bounded blocker hid:
- **`:wat::core::define` death (arc 241.11 HARD CUT).** 10 deftest files (~57
  helpers) in `crates/{wat-telemetry,wat-telemetry-sqlite,wat-lru}/wat-tests/` +
  `wat-tests/counter-service-process-N3.wat` still use the retired `define`
  call-head form → their `:test::` helpers never register → "unresolved
  reference." Broken since arc 241; the channel collapse just surfaced it.
  **Fix: a define→defn migration sweep (next stone).**
- **`HolonKey.wat` HolonAST/WatAST type error** (`:wat::holon::Atom` expects
  `HolonAST`, got `WatAST`) — a distinct pre-existing wat-lru type-rot, not
  define-related.

These are NOT make-channel regressions (the failing files are ones the cascade
either didn't touch or only touched channel-wise; the errors are define/type, not
channel). The make-channel stone stands green on its own terms.
