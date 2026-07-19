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
| **`Sieve`** | `wat.query` (general — it's rete-over-a-page, not telemetry-specific) | the pure filter spec: **`Sieve = Predicate \| Rules`** (a union enum — one op takes one `Sieve`) |
| **`Sieve::Predicate`** | — | a pure predicate **fn-form** over one seed log: `(fn [log] -> :bool …)` |
| **`Sieve::Rules`** | — | user-supplied **`[defs <- Vec<defrecord>  rules <- Vec<Rule>]`** — the defs decode the opaque message into typed facts (or `read-foreign` → dynamic facts); the rules select. The full rete / chaos-engine form. |

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
