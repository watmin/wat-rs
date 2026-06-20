# DESIGN — Stone P12a: the EXPLAIN substrate (`fire-rules-explain` + the support index)

First strike of P12 (the guiding light). Builds the **native substrate** the wat `explain` walk (P12b) reads.
RED-gated by its own probe; the P12 north-star (`tests/probe_arc278_P12_explain_walk.rs`) greens at P12b.

## What it delivers
An **opt-in** diagnostic fire that retains the support graph and records, for each derived fact, the token that
produced it (and the rule). The fast `fire-rules'` is **untouched** — it stays lean (clears beta, no index).
This is the opt-in principle made real (see DESIGN-STONE-P12, "diagnostics are OPT-IN"): the diagnostic costs
nothing on the hot path and is re-derivable from the stored `{facts, rules}` on demand.

## The contract decision (pinned, four-questioned → A)
`fire-rules-explain` returns a **new ephemeral type `:wat::rete::Explained`**, NOT a mutated `Session`:

```clojure
(:wat::Record::def :wat::rete::Explained
  [session <- :wat::rete::Session
   support <- :wat::core::PersistentMap])   ;; derived-fact → producing-(rule, token); see "support shape"
```

- `(:wat::rete::fire-rules-explain <session>) -> :wat::rete::Explained`
- **The base `Session` is byte-for-byte unchanged.** The fast path and the `{facts, rules}` snapshot pay
  nothing; the diagnostic payload rides only in the opt-in return. (Four-questions, hard-constraint-first: B —
  a `support-index` field on `Session` — fails Honest, the optional-in-practice-field smell
  [[feedback_optional_is_a_smell]], and Simple, the 7-field ripple through kernel + oracle. A is unanimous.)
- **`Explained` is EPHEMERAL** — re-derived per explain, never serialized. No EDN round-trip, no `from-edn`,
  no revive. The snapshot stays exactly `{facts, rules}`; `Explained` is what you get when you *force* it in
  explain mode.

### support shape (the index)
`support` maps a **derived fact → its producing `(rule-name, Token)`**. The Token already carries
`matches = PV<(fact, alpha_id)>` (the condition-edges = the support chain) + `bindings`. v1: **first producing
token wins** (a fact derived two ways → the first; multi-derivation fan-in is a named follow-on, per the P12
DESIGN). Concretely the value is a 2-tuple `(rule-name-String, Token)` so P12b's walk reads both the rule (for
`Why.rule`) and the chain (for `:via` recursion).

## The seam (rooms — exact, already grounded)
1. **`production_pass` (`src/rete/kernel.rs:952`)** — the recording point. It already holds `tok` (the producing
   token, with `matches`) AND `rule_name` (line 913) exactly where it builds `derived` and pushes the fact
   (`:953`). The explain variant additionally records `derived → (rule_name, tok)` into the index. The data is
   already in hand — the fast path simply discards it.
2. **The beta-clear (`kernel.rs:986` `fire_once_session`, `:1577` `fire_fixpoint_delta`)** — the fast path
   clears beta at freeze (the P11 line-rate win). `fire-rules-explain` runs the **same fixpoint without that
   final clear** + threads the index. The fast path's two clears stay exactly as they are.
3. **`to_persistent` / freeze** — `Explained` is built at the end of the explain fire: `{session: <frozen
   Session, same as fast path>, support: <the index as a wat PersistentMap value>}`.
4. **Native entry + registration** — `eval_fire_rules_explain` registered as `:wat::rete::fire-rules-explain`
   (mirror the `fire-rules'` registration: runtime dispatch arm + check TypeScheme `Session -> Explained` + mod
   note). `Explained` registered as a builtin Record type (mirrors the other rete records in `rete.wat`).

## Blast radius (bounded)
- `src/rete/kernel.rs` — an explain-mode fixpoint (reuse `fire_fixpoint_delta`'s body; parameterize the
  final-clear + index-recording, or a sibling fn) + `production_pass` index recording (explain path only) +
  the `Explained` build. The fast path's functions stay behavior-identical.
- `wat/rete.wat` — the `:wat::rete::Explained` Record def + the `fire-rules-explain` public verb wrapper (one
  line over the native, like `fire-rules`). **NO change to `Session`, `fire-rules'`, `fire-rules-spec`, or any
  existing fire path.** ⚠ rete.wat is the differential oracle — additive only; the existing differential
  (deep-cascade + P4a/P4c) must stay green (this stone adds a path, changes none).
- A new probe `tests/probe_arc278_P12a_explain_substrate.rs`.
- **NOT** the wat `explain` walk (P12b), **NOT** the `Why`/`WhyVia` records (P12b), **NOT** `:met` (P12c),
  **NOT** the base `Session` type, **NOT** `from-edn`/revive of `Explained`.

## The RED probe (P12a's gate — write FIRST, RED at HEAD)
A probe that fires the cold-and-windy cascade in explain mode and reads the index — proving the producing token
is captured with its chain, WITHOUT needing the wat walk yet:
1. `(:wat::rete::fire-rules-explain session)` returns an `Explained` (not a Session). RED at HEAD (UnknownFn).
2. `(:wat::rete::Explained/session result)` round-trips the derived facts identically to `fire-rules'`
   (the diagnostic mode does not change WHAT is derived — same closure). Assert via `query`/`collect-derived`
   count == the fast path's.
3. The `support` map, keyed by the derived `ColdAndWindy` fact, yields a token whose `matches` has length **2**
   (the Temperature + WindSpeed support edges). This is the chain, captured. Assert the scalar 2.
   (Accessors for reading the token's matches: reuse what P11's kernel tests use; if a wat-level reader is
   missing, that gap is P12a's to expose minimally — surface it, don't hand-wave.)

## Build-step #1 (verify before building)
Confirm `fire_fixpoint_delta` (`kernel.rs:1191`) is cleanly parameterizable for "retain beta + record index"
without forking 200 lines — if the explain path can be the same body with a flag/closure for the final-clear
and the production-pass recording, do that (one engine, two modes). If it would require a near-total copy,
STOP and surface — a copy is a differential-drift hazard (two fixpoints to keep in sync).

## STOP triggers
1. **STOP if `Session` would need a field** to make any of this work. The contract is A — `Explained` carries
   the payload; `Session` is untouched. If something forces a Session field, the design assumption broke —
   surface it.
2. **STOP if the fast `fire-rules'` / `fire-rules-spec` behavior changes** (differential goes red). This stone
   is purely additive; the existing differential is the guard.
3. **STOP if recording the index requires `production_pass` to eval/mutate beyond what the fast path does**
   (it must be pure record-keeping of data already in hand). No new RHS evaluation.
4. **STOP if `Explained` needs EDN round-trip / `from-edn`** to satisfy the probe — it's ephemeral; if the
   probe seems to need persistence, the probe is wrong, not the design.

## Four-questions
- **Obvious?** YES — opt-in diagnostic fire returns "the session plus why it derived what it did."
- **Simple?** YES — additive native path reusing the fixpoint; one new ephemeral record; the data is already in
  hand at `production_pass:952`.
- **Honest?** YES — the fast path is provably unchanged (differential); diagnostics are a distinct type, not a
  smell-field; the chain captured is the *real* one the fire produced.
- **Good UX?** YES — `explain` (P12b) will take an `Explained`, so you cannot explain a session you didn't opt
  into; the mistake is unrepresentable.
