# DESIGN — `sift`: server-side log filtering, the chaos engine's first concrete form

> **Origin (builder, this session):** envisioning the telemetry sink as a *networked* service — "if we wanted
> server-side filters, we could run the rete query tooling… the user supplies the rules, the server spawns a
> thread to do the query… only what traverses the wire is the records actually desired… **this is the first chaos
> engine**." R25 `MACHINA CHAOS DOMAT` concretized: rules imposing order on a flood of facts, in a *paged* form.
> Names intueri-cast + **ratified** this session (`sift-logs`/`Sieve`/`Predicate`/`Rules`); the `Rules`-not-`RuleSet`
> sub-call ratified by the builder. This is the **T2 / R0 tier — after Stone B** (the opaque sink is its floor).

## What it is
A client wants only the logs it cares about *without* shipping every row over the wire to filter client-side. So
the client submits a **pure filter spec** (`Sieve`); the server runs it over a page and returns only the survivors.
Server-side filtering. Only the desired records cross the wire back. It **dogfoods telemetry to measure rete**
(measure-first) and is the chaos engine's first *batch/paged* incarnation — the streaming `Session`-as-state R0 is
the fuller form; this runs first.

## The ratified vocabulary
| name | namespace | what it is |
|---|---|---|
| **`sift-logs` / `sift-metrics`** | Journal op (beside `query-logs`/`query-metrics`) | query a namespace + window, **filtering server-side**, returning only survivors |
| **`Sieve`** | `wat.query` (general — it's rete-over-a-page, not telemetry-specific) | the pure filter spec: **`Sieve = All \| Predicate \| Rules`** (a union enum — one op takes one `Sieve`) |
| **`Sieve::All`** | — | **no filter** — a match arm that SKIPS the worker and just hydrates the page (today's `query-*`, the fast path preserved). Distinct variant, NOT a pass-all `Predicate` — won on *Honest* (a pass-all predicate lies: it spins the worker to filter nothing). |
| **`Sieve::Predicate`** | — | a pure predicate **fn-form** over one seed log: `(fn [log] -> :bool …)` |
| **`Sieve::Rules`** | — | user-supplied **`[defs <- Vec<defrecord>  rules <- Vec<Rule>]`** — the defs decode the opaque message into typed facts (or `read-foreign` → dynamic facts); the rules select. The full rete / chaos-engine form. |

> **★ RULING (builder, 2026-07-19) — the reader interface CONVERGES to `sift-{logs,metrics}`; `query-*` is subsumed.** `query` IS `sift` with a pass-all sieve (grounded: `query-logs` `journal.wat:164-192` is the scan→hydrate `foldl` with no filter; `sift` is that loop + the filter). So the endpoint is ONE reader family. **`Sieve::All` is a distinct variant** (agreed — Honest: skips the worker, no read-string/verify/eval/apply for a no-op). **Order (ratified): `Predicate` (task #5) → `Rules` (task #6) → `Sieve::All` + ANNIHILATE `query-{logs,metrics}` — the LAST stone**, delivered together (COMPONENDO DELEO / R48 — `query-*` is subsumed scaffolding). Do NOT widen the Predicate strike with `All`; it is the tracked campaign tail (done-is-done). `sift-metrics` with `Sieve::Rules` is how metrics AGGREGATE (rete accumulators, R4/stone-8) — the two-op family covers filter AND aggregate; the `Sieve` carries the difference.

Reads on the line — **the enum IS the interface** (R28 decomplection: the surface says "a `Sieve`, either form"):
```clojure
(sift-logs journal
  (SiftLogsRequest :namespace "…" :time-lo 0 :time-hi T :limit 1000 :cursor :None
    :sieve (Sieve::Predicate (fn [log] -> :bool …))))     ; the simple form
    :sieve (Sieve::Rules :defs [ … ] :rules [ … ])        ; the chaos-engine form
```

## The execution model (load-bearing — the builder's correction, kept literal)
- Slurp a page of rows from the store (cap ~1000). Build a lazy seq; stream it through the filter; the output seq
  is a **different, smaller** size — the filter doing its job.
- A rete **`Session` = the rules compiled, working memory EMPTY** (rules defined, no base facts).
- **Fire ONCE PER RECORD with exactly ONE seed** — that log, alone in WM — collect its `Deduction`s (pass) or
  none (drop), then **RESET to empty before the next record**. *One log never poisons another;* each fire is a
  pure `fn(rules × one-seed)`, isolated + disposable (`RENASCOR NON RETRACTO` at the record grain). The Session
  shell (rules) is compiled once and reused; only the seed varies, and it is alone each fire.
- **Alpha-only is STRUCTURAL, not a policy:** one base fact per fire ⇒ no second base fact to beta-join ⇒
  cross-record joins are *impossible by construction*, page boundaries or not. The seed still cascades its own
  Lemmas (`Record → Lemma* → Deduction`, single-lineage, within its own fire).
- Runs on a **throwaway capacity-one worker (a bracket)**: spawn, run the pure filter over the page, reap. The
  sink's serve loop never blocks; purity makes the worker disposable (no cleanup, no shared state).

## The wire truth (the corrected model — a fn IS EDN)
Both spec forms are **pure homoiconic DATA that cross the wire** — a fn-*form* is EDN/AST exactly as a rule-form
is (it is how a forked process gets its program at all); this is **not** a live closure. 293.W governs a live
impure *value* (a compiled closure with a captured env, a `Peer`), not quoted source. So there is **no
local-vs-networked split** between the two forms — both cross, both eval on the server. The axis that survives is
**purity**, and it is a *check*: the server verifies the spec pure (`pure?` / `deterministic?`) and eval's it in a
sandbox (`run-sandboxed-ast` under a `restricted-to` whitelist) — untrusted-but-verified-pure code, safe. The
`Predicate` form is the simpler primitive (apply-per-record, no Session); the `Rules` form is the full rete. Phase
the `Predicate` form first (simpler to wire), the `Rules` form second — for simplicity, **not** locality.

## The convergence — this is what Stones A + B were the floor for
- **B** stores `Log.message` *opaque* (EDN-text String) → the sink never decodes → arbitrary callers, no DoS.
- The **filter thread is a consumer** — "decoding foreign data is the consumer's problem"
  (`feedback_sink_is_opaque_store_consumer_decodes`). It decodes on demand: user supplies `Sieve::Rules :defs` →
  register them → `read` the opaque message **typed** → typed facts; no defs → **`read-foreign`** (Stone A) →
  `ForeignRecord` facts → rules match by key.
- **Purity is the whole validity** (R5/R18): pure rules + pure facts ⇒ the filter is a total function of
  `(facts × rules)` ⇒ safe to fling on a throwaway thread, re-fire freely, discard — no TMS. Clara *can't* (impure
  RHS). The scope-reduction we imposed (purity) is exactly what makes a disposable per-query rete worker sound.
- A holon fact carries its Hologram → a rule can do a **VSA op** (similarity/residual) mid-fire — R4's
  "the novel half has a seam waiting": rules over *resemblance*, not just equality.

## Scope + sequencing
- **Floor:** Stone B (opaque `Log.message`) must land first — the sift filter decodes it on demand.
- **Then:** `sift-logs`/`sift-metrics` as a Journal op; the `Sieve` enum in `wat.query`; the `Predicate` form
  first (per-record pure-fn apply), the `Rules` form second (per-record rete fire), both on the throwaway bracket.
- **OUT (this doc):** the streaming `Session`-as-state R0 (the fuller chaos engine — a live WM across messages);
  page-local beta joins (rejected — one-seed-per-fire makes alpha-only structural); client-side filtering (the
  whole point is server-side).

*Realization-shaped (R25 concretizing into its first buildable form, the VSA seam lit) — the song is the builder's
to hand, not the record's to mint.*

## Predicate-form strike — GROUNDED (scout, on the reclaimed floor; HEAD `f11e64db`)
The first strike (the `Sieve::Predicate` form). All `file:line` verified this session; everything uses the
**de-primed** names (`:wat::telemetry::Journal`, `:wat::query::`, `:wat::sqlite::`).

- **Add the Journal op** (`wat/telemetry.wat`, beside `query-logs` at ~:147-173): a `Journal::SiftLogsRequest`
  (the same window/page fields as `QueryLogsRequest` — namespace/time-lo/time-hi/limit/cursor — **plus** `sieve <-
  :wat::query::Sieve`) + a `Journal::SiftLogsResponse` shaped exactly like `QueryLogsResponse` (`:Success [logs,
  cursor]` / `:Transient` / `:Fatal`) + the method sig in `:features`. Purely additive; the surface already ships
  its protocol across a fork (`:messages` surface-forms carrier).
- **The insertion point** (`wat/telemetry/journal.wat`): `query-logs`'s scan→hydrate `foldl` (~:181-186) — scan the
  store, hydrate each `Row/data` via `edn::read` → `Log`. `sift-logs` is that loop with the filter added:
  `(if (predicate log) (conj acc log) acc)`. `query-metrics` is the twin.
- **`Sieve` lives in `wat/query.wat`** (the rete-as-datalog home, query.wat:9-10) as a `defenum :wat::enum::Pure`:
  `:Predicate [pred <- :wat::core::String]` | `:Rules [defs <- (Vector …) rules <- (Vector …)]`.
- **★ THE CONTRACT DECISION (pinned, grounded): the `Predicate` field is a `:wat::core::String` of EDN source
  text — NOT `:wat::WatAST`.** A `WatAST` field serializes to **opaque-nil** across a process fork (general
  `edn::write` renders an AST as nil, `src/edn_shim.rs:445-446`) — it would silently null the predicate on the wire.
  Carry it as an EDN-source String (the exact opaque-String pattern `Log.message` uses). The server rebuilds it:
  `(:wat::core::read-string pred-src)` → **verify** `(and (:wat::rete::pure? form) (:wat::rete::deterministic? form))`
  (the two-axis gate, `src/rete/purity.rs:10`, verbs at `:421-438` — verify the QUOTED form BEFORE eval) →
  `(:wat::eval-ast! form)` → a `:wat::core::fn` value → `(:wat::core::apply -> :bool pred-fn log [])` per record
  (the one-arg fn-value apply fast-path, `src/runtime.rs:8422-8424`).
- **Sandbox reality (grounded):** there is NO one-call "eval a pure fn under a `restricted-to` whitelist" primitive.
  `run-sandboxed-ast` (`wat/kernel/hermetic.wat:89`) is a whole-program/process runner returning a `RunResult` —
  heavier, and it IS a fork. The pragmatic per-record safety is **`pure? ∧ deterministic?` verification + plain
  `apply`** (a rejected predicate → a clean located error, no-hidden-failures). If OS-isolation is later wanted,
  reconcile the sandbox fork with the bracket so you don't double-fork.
- **The throwaway worker:** no dedicated "capacity-one" bracket form. Configure the locus with `runner-count 1`
  (`wat/spawn.wat:97-101` thread / `:79` process) and pass it to `map`/`map-worker` (`wat/bracket.wat:531`, which
  does spawn→run→**reap** RAII, revoking grants before return, :574-586). Or the lower-level single-shot
  `spawn-program'` peer (bracket.wat:8) if the pool coordinator is more than needed.
- **RED gate:** a page of stored Logs filtered by a pure predicate returns ONLY the survivors (e.g. `(fn [log] ->
  :bool (= (Log/level log) :error))` over a mixed page → only the errors); an **impure** predicate is REJECTED
  (the `pure?`/`deterministic?` gate raises a clean located error, not a silent pass — the no-hidden-failures floor).
- **Rules form (later, task #6) — note:** `Sieve::Rules` uses rete: `(:wat::rete::compile rules)` → Session (rules
  compiled, WM empty, `rete.wat:796`) → per record `(-> (compile rules) (insert one-log) fire-rules)` →
  `query-by-type-string` for survivors → discard. One `insert` per fire ⇒ alpha-only is structural. `defs` decode the
  opaque message into typed facts (or `read-foreign` → dynamic facts); holon facts admit VSA ops mid-fire.

**Flags for the strike:** (1) the String-of-EDN-source field type is the fork-safety crux — do NOT use `WatAST`;
(2) `pure?`/`deterministic?` are `:wat::rete::` verbs (the sift service takes a rete dependency — fine, the Rules
form needs rete anyway); (3) "sandbox" = verify-then-apply, not a ready sandboxed-apply; (4) capacity-one =
`runner-count 1`, not a named form.
