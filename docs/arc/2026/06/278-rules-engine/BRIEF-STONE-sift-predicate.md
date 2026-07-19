# BRIEF — Stone 2: the sift Predicate delivery (the chaos engine's first paged form)

> Executor tier: sonnet shadowdancer. Orchestrator weighs by its own re-run; commits.
> Design ratified: DESIGN-sift-server-side-filter.md ("Predicate-form delivery — DESIGNED"). Stone 1
> (`:wat::core::ast->source`) is LANDED (`037ddf88`). The full server chain is PROVEN to compose (the
> disconfirming probe `scratchpad/probe-sift-chain.wat` → `SIFT-CHAIN-OK`): captured `(fn …)` → ast->source →
> String → read-string → unwrap → pure?/deterministic? → eval-ast! → apply(record) → :bool. **Stone 2 is
> WIRING, not invention** — wire that chain into a Journal op, behind an organic-UX capture macro.

## The work (one paragraph)
Deliver server-side log filtering. The client writes a real `(fn [log] -> :bool …)`; a **`sieve-pred` macro**
captures it and `ast->source`s it into a `:wat::query::Sieve::Predicate` value (a `String` field — the user
NEVER types a string). That `Sieve` rides in a `Journal::SiftLogsRequest`; the `journal'` service's **`sift-logs`
op** compiles the predicate ONCE (read-string → unwrap → verify pure?∧deterministic? → eval-ast!) and applies it
per row inside the scan→hydrate `foldl`, returning only the survivors. An impure predicate is REJECTED (a clean
located error — the no-hidden-failures floor). `sift-metrics` is the mechanical twin.

## Read in order (the rooms, each with why)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-sift-server-side-filter.md` — the ratified design (the vocabulary,
   the execution model, the server chain, the four-questions).
2. `scratchpad/probe-sift-chain.wat` — the PROVEN server chain (copy its exact verb sequence: `read-string →
   first(ast->children …) → pure? → deterministic? → Result/expect(eval-ast! …) → apply -> :bool pfn record []`).
3. `wat/rete.wat:1954-1996` (`defrule` macro) — the EXACT `defmacro` model: params typed `:wat::WatAST`, body
   builds the expansion via quasiquote `` ` `` / `~` / `~@`. The `sieve-pred` macro mirrors this.
4. `src/macros/eval.rs:615-622` (`is_pure_total`) — the macro-expand allow-list. It has `write-forms`/`ast-name`/
   `ast->children` but **NOT `ast->source`**. ADD `| ":wat::core::ast->source"` here — else the macro cannot call
   it at expand time. (grounded gap; it's pure ∧ deterministic, belongs beside its siblings.)
5. `wat/query.wat:26-95` — where `:wat::query::` records/enums live (StoredRow, Page, the `Store` surface, the
   `defenum :wat::enum::Pure` recovery enums). The `Sieve` enum + the `sieve-pred` macro go in this file.
6. `wat/telemetry.wat:115-193` (the `Journal` surface, `:nature :Peer'`) — `QueryLogsRequest`/`QueryLogsResponse`
   in `:messages` + `query-logs`/`query-metrics` in `:features`. Mirror them for sift (add `:sieve` to the request).
7. `wat/telemetry/journal.wat:164-192` (`query-logs` op) — the EXACT twin `sift-logs` extends: scan → `match resp`
   → `foldl(hydrate via edn::read)`. `sift-logs` = this + the compiled predicate applied per row.
8. `tests/services/probe_arc278_journal_logs_on_process.{rs,wat}` (or the nearest live `journal'` service test) —
   the integration-test harness (spin a `journal'`, write logs, query) to copy for the RED gate.

## Implementation sketch (fill it; do not invent the shape)
**(a) `wat/query.wat` — the Sieve enum (`:Predicate` ONLY; `:All`/`:Rules` are later stones):**
```clojure
(:wat::core::defenum :wat::query::Sieve :wat::enum::Pure
  (:Predicate [pred <- :wat::core::String]))     ;; pred = the ::-source of a (fn [log] -> :bool …)
```
**(b) `wat/query.wat` — the `sieve-pred` capture macro (organic UX — user writes the fn, never a string):**
```clojure
(:wat::core::defmacro :wat::query::sieve-pred
  [fn-form <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [src (:wat::core::ast->source fn-form)]
    `(:wat::query::Sieve::Predicate ~src)))
```
**(c) `src/macros/eval.rs:615` — add `ast->source` to `is_pure_total`** (one match arm).
**(d) `wat/telemetry.wat` — Journal surface: add `Journal::SiftLogsRequest` (QueryLogsRequest fields + `sieve <-
:wat::query::Sieve`) + `Journal::SiftLogsResponse` (mirror QueryLogsResponse: `:Success [logs cursor]` |
`:Transient [err]` | `:Fatal [err]`) to `:messages`; add `sift-logs` (+ `sift-metrics` twin) to `:features`.**
**(e) `wat/telemetry/journal.wat` — the `sift-logs` op (mirror `query-logs`; compile ONCE, apply per row):**
```clojure
(sift-logs [s req]
  (:wat::core::let
    [pred-src (:wat::query::Sieve::Predicate/pred            ;; the ::-source string
                (:wat::telemetry::Journal::SiftLogsRequest/sieve req))
     pform    (:wat::core::first (:wat::core::ast->children (:wat::core::read-string pred-src)))
     ;; REJECT impure — the no-hidden-failures floor (return ::Fatal with a Fault, do NOT silently pass):
     …verify (and (pure? pform) (deterministic? pform)); if not → SiftLogsResponse::Fatal (a Fault message)…
     pfn      (:wat::core::Result/expect (:wat::eval-ast! pform) "sift-logs: eval predicate")
     …scan (identical to query-logs)…]
    ;; the foldl body: (if (:wat::core::apply -> :wat::core::bool pfn log []) (conj acc log) acc)
    …))
```
Compile the predicate ONCE (outside the foldl); apply per row INSIDE it.

## Blast radius (bounded)
`wat/query.wat` (Sieve enum + sieve-pred macro) · `src/macros/eval.rs` (one allow-list arm) · `wat/telemetry.wat`
(Journal surface: 2 request + 2 response messages + 2 methods) · `wat/telemetry/journal.wat` (sift-logs +
sift-metrics ops). No changes to Store, to write-forms, to the existing query-* ops (they coexist until the
`Sieve::All` tail annihilates them — a LATER stone).

## Out of scope = rejected (affirmative cuts)
- **`Sieve::All` and `Sieve::Rules`** — later stones (the RULING order: Predicate → Rules → All+annihilate). Define
  `Sieve` with `:Predicate` only; it grows later.
- **The throwaway `runner-count 1` worker** — Stone 2 runs the filter INLINE in the op (compile-once, apply-per-row);
  correct + sufficient to lay the architecture. The non-blocking worker is a tracked follow-on (it matters for the
  Rules form / not blocking the serve loop), NOT this stone.
- **query-* annihilation** — the tail stone, not now.

## STOP triggers (halt + surface; do not improvise)
- **STOP-1:** if the `sieve-pred` macro cannot call `ast->source` even after adding it to `is_pure_total` (some
  other expand-time gate), STOP and report the exact rejection — do not route around it.
- **STOP-2:** if `eval-ast!`/`apply` inside the `journal'` service op cannot resolve the hydrated `Log` type in
  scope (the predicate reads `Log/level`), STOP and surface it — the fix is scope threading, not a workaround.

## The RED gate (install + make green)
Two fixtures:
1. **Macro unit** (`tests/rete/probe_arc278_sieve_pred.{rs,wat}`, `call_beside` idiom — copy
   `tests/rete/probe_arc278_ast_to_source.{rs,wat}`): `(:wat::query::sieve-pred (fn [log <- :T] -> :bool …))`
   expands to a `Sieve::Predicate` whose `pred` string contains `"::"` (verbatim) and round-trips through
   read-string to the same fn-form.
2. **Op integration** (`tests/services/probe_arc278_sift_logs.{rs,wat}`, copy the live `journal'` service test):
   write a mixed page of Logs; `sift-logs` with a pure predicate (e.g. `level = :error`) returns ONLY the
   survivors; `sift-logs` with an IMPURE predicate returns `::Fatal` (rejected, not a silent pass).

## Expectations
| what | command | expected |
|---|---|---|
| both gates green | `cargo test --release -p wat sieve_pred` + `… sift_logs` | pass |
| nothing else breaks | `cargo nextest run --release` (Summary line) | 0 new failures |
Runtime: ~10–15 min. Trap-doors: the `is_pure_total` gap (STOP-1); Log-type scope in eval-ast! (STOP-2).

## Report back (raw facts, not narrative)
(1) both gate results (paste the test output); (2) the nextest Summary line proving 0-new; (3) the exact
files+line-ranges touched; (4) any STOP-trigger surfacing. Do NOT commit.
