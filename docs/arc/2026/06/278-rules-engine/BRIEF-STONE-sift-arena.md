# BRIEF — the sift Predicate ARENA: trial by combat (two universes, foreign reader, brutal)

> Executor tier: sonnet shadowdancer. Orchestrator weighs by its own re-run; commits.
> This proves the sift Predicate EARNS ITS KEEP on the real use case — arbitrary services flooding a shared
> journal with their OWN-universe records, a consumer querying them via the FOREIGN READER, guaranteed to not hold
> their types. It is CloudWatch Insights in wat, and the trial that guides the Rules form. Slow is smooth; strike
> to kill; prove it, do not simulate it.

## Two findings already grounded (do not re-derive)
1. **The purity fence rejects any read-foreign predicate** — `intrinsic_meta` (`src/rete/purity.rs:66-190`) has no
   `:wat::edn::` entry, so `read-foreign`/`ForeignRecord/get` default-deny. Proven: `scratchpad/probe-foreign-pred-purity.wat`
   → `FOREIGN-PRED-REJECTED`. **Part A fixes this.**
2. **The universe guarantee is REAL** — `scratchpad/probe-universe-guarantee.wat` → `class="prod::Alert"`,
   `severity="high"`: a PROCESS consumer that never compiled `:prod::Alert` decodes it via `read-foreign` (a
   `ForeignRecord`, because it genuinely lacks the type). The arena rests on this.

## PART A — the fence fix (prerequisite; unblocks the consumer's predicate)
Read `src/rete/purity.rs:66-74` — `intrinsic_meta` already classifies whole PURE namespaces by prefix
(`:wat::core::string::`, `:wat::core::regex::` → pure ∧ deterministic). The entire `:wat::edn::` namespace is pure
data transforms — parse/serialize/navigate, no IO, no entropy (grounded verb set: `read`, `read-foreign`, `write`,
`write-pretty`, `write-json`, `write-json-natural`, `ForeignRecord/get`, `ForeignRecord/class`,
`ForeignVariant/variant`, `ForeignVariant/enum-class`, `ForeignVariant/fields`).
**FIX:** add, beside the string::/regex:: prefix rule (~line 72):
```rust
if head.starts_with(":wat::edn::") {
    return Some(OpMeta { pure: true, deterministic: true });
}
```
Root-level, not a per-verb hand-list the next foreign verb slips past — the whole namespace is pure by nature.
**RED gate A** (`tests/rete/probe_arc278_foreign_pred_purity.{rs,wat}`, copy the `probe_arc278_accessor_purity`
idiom — `call_beside` → bool): (1) a foreign-reader predicate
`(fn [log <- :wat::telemetry::Log] -> :bool (= (ForeignRecord/get (read-foreign (Log/message log)) :severity) "high"))`
passes `pure?` ∧ `deterministic?`; (2) GUARD: the SAME predicate with an impure op in the body (a `println`) is
STILL rejected (the edn ops are pure, but the impure op fails — conditional purity, not blanket-allow). The scratch
seed is `scratchpad/probe-foreign-pred-purity.wat` (currently RED — flips green after the fix).

## PART B — the arena (the two-universe flood-and-sift, PROCESS-tier, guaranteed foreign)
Read in order: `wat/telemetry.wat:115-212` (the `Journal` surface — `:nature :Peer'`, `:messages` = defrecords +
defenums, `:features` = kebab ops; the SHAPE both new services copy); `wat/telemetry/journal.wat:135-283`
(the op impl pattern + the Stone 2 `sift-logs`); `tests/services/probe_arc278_journal_service_sqlite_on_process.wat`
(multi-service-on-process + **grant-before-dial** via `:locus (:wat::spawn::process/post-spawn (fn [pl] (store/grant …)))`);
`tests/services/probe_arc278_sift_logs.wat` (calling `Journal/sift-logs` + the `SiftLogsRequest`/`Sieve`/`sieve-pred`
shape); `scratchpad/probe-universe-guarantee.wat` (the read-foreign-across-universes pattern; class string = `"prod::Alert"`).

**The architecture (one fixture, `tests/services/probe_arc278_sift_arena.{rs,wat}`):**
- **Producer surface + service** — `:prod::Producer` (`:nature :Peer'`), `:messages` carry the producer's OWN log
  payload types (arbitrary records — nothing forces Op/Reply): `:prod::Alert [severity <- :String  code <- :i64]`,
  `:prod::Flow [proto <- :String  bytes <- :i64]`, `:prod::Query [rows <- :i64]` — PLUS the op pair
  `FloodRequest [count namespace]` / `FloodResponse :Pure [:Done [written <- :i64]]`. `:prod::producer'` `:satisfies`
  it, `:peers [:wat::telemetry::Journal]`. Its `flood` op: build `count` `Log`s cycling the 4 shapes deterministically
  (`mod i 4` → Alert-high / Alert-low / Flow / Query), each `Log.message = (edn::write (:prod::Alert :severity "high" …))`
  etc., then `Journal/write-logs` (one batch or chunked); reply `Done written`.
- **Consumer surface + service** — `:cons::Consumer` (`:nature :Peer'`), `:messages` = `SiftRequest [namespace]` /
  `SiftResponse :Pure [:Count [n <- :i64]]`. `:cons::consumer'` `:satisfies` it, `:peers [:wat::telemetry::Journal]`
  — **NEVER peers Producer** (this is the guarantee: its registry lacks `:prod::*`). Its `sift` op PAGES through the
  journal: a TCO cursor-loop helper calls `Journal/sift-logs` with a small `:limit` (e.g. 50), accumulating survivor
  count across pages until `next-cur` is `None`. The sieve is a `sieve-pred` over a FOREIGN predicate, class-guarded
  (heterogeneity — some payloads LACK `:severity`; `ForeignRecord/get` on a missing key ERRORS, so guard first):
  ```clojure
  (:wat::query::sieve-pred
    (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
      (:wat::core::let [fr (:wat::edn::read-foreign (:wat::telemetry::Log/message log))]
        (:wat::core::if (:wat::core::= (:wat::edn::ForeignRecord/class fr) "prod::Alert")
          (:wat::core::= (:wat::edn::ForeignRecord/get fr :severity) "high")
          false))))                                  ;; explicit `if` short-circuit — do NOT rely on `and`
  ```
- **Orchestrator** (`:user::compute`, the circuit builder): start `mem-store'` + `journal'` (process, shared) +
  `producer'` + `consumer'` (process; grant-before-dial where a child dials the journal/store). Call `producer/flood`
  (block), then `consumer/sift` (block). Return the count.

**The RED gate (the teeth):** flood N=240 logs (target = the 60 Alert-high) into a namespace; the consumer, which
CANNOT hold `:prod::*`, pages through with the read-foreign class-guarded predicate and returns **exactly 60**. The
exact count across many pages, via the foreign reader, on a process consumer that provably doesn't know the type,
IS the proof. PROCESS-tier (the guarantee is a process property — separate registries).

## Out of scope = rejected
- Thread-tier universe isolation (a thread shares the registry — the guarantee is process-only; sift's thread≡process
  parity is already proven in Stone 2's gate).
- The `Sieve::All`/`Sieve::Rules` variants (later stones).

## STOP triggers (halt + surface — findings, not workarounds)
- **STOP-1:** if a producer child cannot construct its own `:messages` types (they don't reach the satisfier's child
  registry), STOP and report — that's a services-as-surfaces finding.
- **STOP-2:** if the class-guarded predicate STILL errors on a `:prod::Flow`/`:prod::Query` (missing `:severity`)
  because the `if` didn't short-circuit, STOP and report the evaluation order (a real semantics finding).
- **STOP-3:** if paging never terminates (cursor never becomes `None`) or drops/double-counts survivors across a page
  boundary, STOP and report (a pagination finding that guides the Rules form).

## Expectations
| what | command | expected |
|---|---|---|
| gate A (foreign-pred purity) | `cargo test --release -p wat foreign_pred_purity` | pass (pred pure; impure-body rejected) |
| gate B (the arena) | `cargo test --release -p wat sift_arena` | pass (exact 60 survivors across pages) |
| nothing else breaks | `cargo nextest run --release` (Summary) | 0 new failures |
Runtime: ~15–25 min. Trap-doors: the `:messages`-types-reach-the-child question (STOP-1); `if` short-circuit (STOP-2);
paging termination/accuracy (STOP-3).

## Report back (raw facts, not narrative)
(1) both gate results (paste); (2) the exact survivor count + how many pages the consumer traversed; (3) the nextest
Summary proving 0-new; (4) files+line-ranges touched; (5) any STOP-trigger surfacing (these guide the Rules form).
Do NOT commit.
