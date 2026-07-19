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

## Predicate-form delivery — DESIGNED (A-vs-B resolved; purity sane; grounded + weighed this session)

The organic UX (ratified by the builder): the user writes a real `(fn [log] -> :bool …)` — **NEVER a string**. The
`Sieve::Predicate` macro (modeled on `defrule`, `wat/rete.wat:1971`, which inserts `(:wat::core::quote …)` into its own
expansion) captures + quotes the fn and `ast->source`s it into a `:wat::core::String` field. The user authors no EDN,
no string.

### Why the field is a `String` (CORRECTED — the prior "WatAST→nil" rationale was WRONG)
A `WatAST` field cannot cross a process wire: the general wire-DECODE crashes on a form's bare symbols
(`edn_shim.rs:1440` — "EDN Symbol: wat has no symbol value type"; wat values are keyword-based, symbols live only in
the AST). *(The earlier claim that `value_to_edn` renders a WatAST as opaque-nil is FALSE — it serializes faithfully
via `watast_to_edn`, but dialect-translates `::`→`.`, which `pure?`+`eval-ast!` reject.)* So the form crosses as
**`::`-source text**, rebuilt with `read-string`. **Loci-agnostic:** a String crosses ANY coordinate (thread == process
— NON-NEGOTIABLE; thread-only is a failure, R31/R32).

### The carry — A-vs-B, resolved + weighed (do NOT re-litigate)
- **A (span→source): INFEASIBLE** — the compilation source buffer is not reachable at macro-expand time (no registry,
  no `span→source` verb, IO denied at expand time; cross-process rules it out too).
- **B (`ast->source` printer): CHOSEN** — resurrect the RETIRED `wat_ast_to_source` (`crates/wat-reader/src/ast.rs:459-466`,
  whose removal note invites `:wat::core::ast-to-source`) as a `WatAST→source` printer. Notation-AGNOSTIC (prints the
  AST's verbatim `::` strings — confirmed `ast-name` returns `:wat::core::fn` verbatim), so it survives the medium-term
  Clojure flip untouched — A's future-stable virtue via a feasible mechanism.

### The server chain (all grounded green, weighed by own re-run)
`read-string → ast->children + first` (**UNWRAP**: `read-string` wraps forms in an outer list — `edn_shim.rs:429`; the
earlier "eval-ast! fails on a fn-form" was a missing-unwrap PROBE bug, not a mechanism gap) `→ pure? ∧ deterministic?`
(now SANE — reads accessor declarations, `17437ffb`) `→ eval-ast!` (returns a `Result`, unwrap) `→ :wat::core::fn value
→ apply` per record → `:bool`. An **impure** predicate is REJECTED by the gate — a clean located error, the
no-hidden-failures floor.

### Stones
- **Stone 1 — `:wat::core::ast->source`** (the enabling primitive; ~60-90 lines, a runtime verb beside `write-forms`).
  RED gate: `ast->source` of a quoted `::`-form → the `::`-source string, round-tripping green through `read-string`.
- **Stone 2 — the delivery**: `Sieve` enum (`wat/query.wat`, `defenum :wat::enum::Pure`: `:All` | `:Predicate [pred <-
  :wat::core::String]` | `:Rules […]`) + the `Sieve::Predicate` macro + `sift-logs`/`sift-metrics` Journal ops (the
  `journal.wat` scan→hydrate `foldl` at `:164-192` + the filter `(if (predicate log) (conj acc log) acc)`, `query-metrics`
  the twin) + the server chain. RED gate: a page filtered by a pure predicate returns only the survivors; an impure one
  is rejected.

### The throwaway worker + Sieve::All + Rules
- **Worker:** `runner-count 1` locus + `map-worker` (`wat/bracket.wat:531`, spawn→run→**reap** RAII). One seed per fire
  ⇒ alpha-only is STRUCTURAL. (No named "capacity-one" form.)
- **`Sieve::All`** (per the RULING above) = the subsuming fast path (skips the worker; hydrates the page — today's
  `query-*`); delivered LAST and annihilates `query-{logs,metrics}` (COMPONENDO DELEO).
- **`Sieve::Rules`** (#6): `(:wat::rete::compile rules)` → Session (WM empty, `rete.wat:796`) → per record
  `insert`/`fire`/query-survivors/reset; one `insert` per fire ⇒ alpha-only structural. `defs` decode the opaque
  message (typed, or `read-foreign` → dynamic facts); holon facts admit VSA ops mid-fire.

**Flags:** (1) the field is a `String` of `::`-source (a `WatAST` field's wire-DECODE crashes — NOT a nil problem);
(2) the macro produces that String via `ast->source` (Stone 1), organic UX; (3) `pure?`/`deterministic?` are
`:wat::rete::` verbs, now sane for accessors; (4) capacity-one = `runner-count 1`, not a named form; (5) DON'T touch
the `::`→`.` dialect — `ast->source` is notation-agnostic and the seam is the medium-term Clojure-flip's job.
