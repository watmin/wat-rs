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
`support` maps a **derived fact → its producing support**, where the value is a typed record (NOT a bare tuple
— no-magic, named accessors P12b reads):

```clojure
(:wat::Record::def :wat::rete::Support
  [rule  <- :wat::core::String       ;; the rule that derived the fact → Why.rule (P12b)
   token <- :wat::rete::Token])      ;; the producing token; token.matches = the support chain → :via (P12b)
```

So `support : PersistentMap<derived-fact, Support>`. The Token already carries `matches = PV<(fact, alpha_id)>`
(the condition-edges = the support chain) + `bindings`. v1: **first producing token wins** (a fact derived two
ways → the first; multi-derivation fan-in is a named follow-on, per the P12 DESIGN). `Support/rule` feeds
`Why.rule`; `Support/token` → `Token/matches` feeds the `:via` recursion.

## The seam (rooms — exact, grounded; corrected from production_pass)
The public `fire-rules'` runs `fire_fixpoint_delta` (the P4b delta engine), NOT `fire_once_session`/
`production_pass`. So the index-recording seam is **inside `fire_fixpoint_delta`'s production-delta loop**, where
the public path actually derives facts.

1. **`fire_fixpoint_delta` production-delta `if !seen` branch (`src/rete/kernel/fire/delta.rs`)** — the recording
   point. The loop fires production nodes on NEW tokens (`d_beta[parent]`); for each `(tok, form)` it builds
   `derived = build_insert_fact(form, &tok.bindings)` and, **`if !seen.contains(&derived)`**, pushes it. That
   `if !seen` branch is exactly where to also record `derived → Support{rule_name, tok.clone()}` — and
   `if !seen` gives **first-producer-wins for free** (v1 semantics). `rule_name` and `tok` are both in scope.
2. **No beta retention needed.** The index **clones the producing token** (with its `matches` chain), so it is
   self-contained: the wat walk reads the index (`fact → Support`), and `token.matches` names the supporting
   facts; a supporting fact that is itself a key in the index → recurse; absent → base/leaf. Nothing reads
   beta. **The `:1577` beta-clear stays exactly as-is** — the fast path is byte-identical. (This corrects the
   P12 DESIGN's "retains beta": the index, not beta, carries the provenance.)
3. **One engine, two modes (build-step #1 → confirmed cheap):** add an optional
   `support: Option<&mut HashMap<Value, (String, Token)>>` param to `fire_fixpoint_delta`. Fast path
   (`eval_fire_rules_native`) passes `None` → zero behavior change. Explain path passes `Some(&mut idx)` and at
   the `if !seen` branch does `idx.entry(derived.clone()).or_insert((rule_name.to_string(), tok.clone()))`. NO
   fork of the 380-line engine → no differential-drift hazard.
4. **`Explained` build** — explain entry calls `fire_fixpoint_delta(&session, sym, Some(&mut idx))`, then builds
   `Explained { session: to_persistent(...) (same frozen Session as fast path), support: <idx → wat
   PersistentMap<fact, Support>> }`.
5. **Native entry + registration** — `eval_fire_rules_explain` (kernel.rs, beside `eval_fire_rules_native`
   :1596) registered as `:wat::rete::fire-rules-explain` at `src/runtime.rs:4012` (the dispatch arm beside
   `":wat::rete::fire-rules'"`) + the check TypeScheme (grep the `fire-rules'` scheme in `src/check.rs`,
   mirror it `Session -> Explained`). `Explained` + `Support` registered as builtin Record types (or defined in
   `rete.wat` as Records like the other rete records — prefer the wat Record def, sibling of `Session`).

## Blast radius (bounded)
- `src/rete/kernel/fire/` — an explain-mode fixpoint (reuse `fire_fixpoint_delta`'s body; parameterize the
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
