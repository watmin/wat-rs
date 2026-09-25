# FINDING — how far declared signatures for intrinsics reach (they don't, yet)

**Measured 2026-09-25**, read-only, at HEAD `4286205ea`. The orchestrator re-verified the load-bearing greps.
The measurement's probes p1–p11 are in the session scratchpad; their results are quoted here.

## Verified by the orchestrator

- **There is no `type_sig` field anywhere** in `src/` or `crates/` (0 hits). Memory said *"type_sig was ruled
  day-one"*: the DESIGN's ruled path is **unbuilt**.
- `FunctionBody::Native` (the DESIGN's locked `FnDef` + native body, `src/value/environment.rs:22-28`) has
  **0 construction sites**.
- **The checker's "declared signatures" are 360 hand-written `env.register(` `TypeScheme` literals** in
  `check.rs`'s `register_builtins` (~:18295–23664), one scheme per name, **so overloads are impossible**.

## What a registry row carries (`src/intrinsic/mod.rs`)

name, handler, kind, `arity`, **`args` + `ret_type` parsed from the Rust `///` `@arg`/`@ret` doc**, plus
prose, examples, purity and so on. **The checker reads only `arity` and `contains` from it.** The doc types
feed reflection, `metadata-of` and render-doc, never the checker.

⭐ **Yet the doc types are already a complete, checker-exact signature:** the passing test
`probe_can_doc_types_reconstruct_the_checker_scheme` rebuilds **449 of 449** schemes exactly from the
`@arg`/`@ret` strings, including all 90 generic ones.

## Census of 582 registry rows

| typed by | rows |
|---|---|
| a `TypeScheme` literal | 449 |
| a hand-written `infer_*` Rust arm | 72, **22 of them in `:wat::kernel::`**, among them every peer intrinsic |
| untyped | 10 |
| special forms | 51 |

The peer intrinsics: `recv`, `send`, `try-send`, `select`, `poll`, `close`, `connect`, `accept`,
`listener`, `allow`, `deny`, `spawn-thread`/`-process`, `signal`, `after`, `peer-process`, `peer-wire?`,
`address-wire?` are all hand rules. `peer-pid` is untyped (`(peer-pid 5)` checks clean). `recv` returns
`(RecvOutcome :- [O])` for **both** vantages today.

**⚠ The docs contradict the checker, and no gate sees it:** the `@arg` for close/recv/send/select says
`(Peer :- [I O])`, but since 255.38 `close` refuses a `Peer` and the others accept a `Spawned`. The
doc-vs-scheme gate skips every row the checker types by hand (`mod.rs:2584`, `None => continue`).

## Can a wat declaration give an intrinsic its signature? No

No bodiless/extern/declare form exists, and a bodiless defclause is refused. The only wat-sourced route is a
wrapper with a body (e.g. `Seqable/seq`), which M0 excludes for `recv`.

## ⭐ Per-receiver-family overloads: the resolution algorithm already exists

**Defclause's check-time dispatch** (`check.rs` ~:5615–5720) is data-driven over `(arg types, ret)`
clauses. Probe p6: clauses on `(Spawned :- [S R])` → i64 and `(Peer :- [S R])` → String send a `Thread`
receiver to i64 and a `Peer` receiver to String. **What is missing for an intrinsic to use it:**

- a bodiless clause-table source (a registry field, or a bodiless declaration form);
- routing: the hand kernel arms `return` before the defclause and scheme lookups.

**Gaps:**

- (a) **no `Vector` covariance under surface satisfaction** (p10), so `select`'s owner signature cannot be
  declared as written;
- (b) **an unresolved receiver silently takes the first clause** (p11), where `recv` refuses today, so clause
  order would decide the vantage;
- (c) targeted diagnostics would degrade;
- (d) `close` needs no overload, only one scheme on `Spawned`.

**What moving the six IPC verbs to declared signatures would delete:** ~567 lines of hand rules, the
`":wat::spawn::Spawned"` string, 18 outcome-type literals, and 2 `"wat::core::Vector"` compares. **What it
would NOT touch:** the must-use lists and remedy, which key on the outcome type. They need a declaration on
the outcome enum, independently.

## Design questions (the builder's), each tied to its measurement

1. **Where does a declared signature live?** Options:
   - the Rust `@arg`/`@ret` docs (already exact for 449/449, but one per row with no overload syntax);
   - a new bodiless wat declaration (none exists);
   - the DESIGN's `FnDef` + `FunctionBody::Native` (0 constructions).
2. Does a row carry **a list** of signatures, resolved by defclause's existing forward match (p6)?
3. `select` over owners: `Vector` covariance under satisfaction, a family-bounded type variable, or keep it
   hand-typed (p10)?
4. An unresolved receiver: first clause wins (p11), or refuse as `recv` does today?
5. Fix the peer `@arg` docs now (they contradict the checker)?
6. `poll`'s permissive self-peer/listener args (p9: `(poll 5 "not-a-listener" ps)` checks): tighten them as
   intended?
7. Keep targeted diagnostics as a pre-check?
8. Must-use: a declaration on the outcome enum (independent of signatures).
9. **Order:** land the recv split on the hand rules first, or make it the first consumer of declared overloads?
