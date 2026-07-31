# BRIEF — `insert'` (native) + `insert-spec` (oracle) + `insert` (delegate)

## The work

`:wat::rete::insert` is the last hot rete verb with no dual. It is interpreted wat doing 7 `Session`
accessors and a 7-field `Session` reconstruction per fact — **87% of the per-fact cost**, and seeding
is 74% of a real workload. Give it the same trio `fire-rules` already has: the wat body becomes the
**oracle** (`insert-spec`), a Rust implementation becomes the **prime** (`insert'`), and the public
`insert` becomes a **one-line delegate**. A RED differential is already in the tree and fails today
with `UnknownFunction: :wat::rete::insert'`; your job is to turn it green.

## Read in order (the rooms, and why you are being sent to each)

1. **`wat/rete.wat:1819-1841` — the `fire-rules` trio.** THE EXEMPLAR. `fire-rules-spec` (the wat
   oracle, pure wat), then `fire-rules` (`:1838`) — a one-line delegate whose whole body is
   `(:wat::rete::fire-rules' session)`. Your three forms mirror these exactly.

2. **`src/runtime.rs:4702-4707` — the dispatch arm.** How `":wat::rete::fire-rules'"` routes to
   `crate::rete::kernel::eval_fire_rules_native`. Note the comment on `:4706`, which states the
   convention as law: *"rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never
   collapsed."* Add the sibling arm for `":wat::rete::insert'"` beside it, with its own
   `// rune:lint(retired-name)` marker in the same style.

3. **`src/rete/kernel.rs:2651` — `eval_fire_rules_native`.** The signature and error style your
   `eval_insert_native` mirrors (arg-count checking, span handling, `EvalBreak`).

4. **`wat/rete.wat:833-844` — `insert` today.** The body you are renaming to `insert-spec`. Its
   header explains *why* it reconstructs the Session (`Record/assoc` returns the base
   `:wat::core::Record`; the typed constructor preserves the concrete type for the checker) — keep
   that reasoning attached to the oracle, since it is still true of the oracle.

5. **`src/runtime.rs:6096-6131` — `keyword_accessor_record`.** The existing, working way to resolve a
   field BY NAME through the class's `RecordDef.field_names` in the TypeEnv. Read it before writing
   your own lookup; the contract below requires this route, not a positional index.

6. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-native-insert.md`** — the measurement, the
   contract, and the affirmative scope cuts.

## ★ THE ONE CONTRACT DECISION

**`insert'` resolves the `facts` field BY NAME, never by positional index.**

A `Session` record value is `class_fqdn` + positional `fields`, with names in the `RecordDef`
(`field_names`). `facts` happens to be index 5 of 7 today. Hardcoding 5 means a future field reorder
writes the **wrong slot silently** — a wrong answer, not a compile error, and the differential may
not catch it if the reordered field has a compatible type. Resolve by name and fail loudly if absent.

Everything else is a structural clone: the other six fields carry through untouched; `:facts` becomes
the conj'd `PersistentVector`; the returned value keeps the `Session` class. `insert` performs **zero
activation** (`rete.wat:828-830` — the WM stays open until `fire-rules`), so touch no memory and walk
no network.

## The three forms

```clojure
;; the ORACLE — the existing body, renamed. Semi-hidden; the beacon of correctness.
(:wat::core::defn :wat::rete::insert-spec [session <- … fact <- …] -> :wat::rete::Session
  …the current body, unchanged…)

;; the PUBLIC VERB — a one-line delegate, mirroring fire-rules at :1838.
(:wat::core::defn :wat::rete::insert [session <- … fact <- …] -> :wat::rete::Session
  (:wat::rete::insert' session fact))
```

plus the `insert'` dispatch arm in `runtime.rs` and `eval_insert_native` in `src/rete/`.

## Blast radius

`wat/rete.wat`, `src/runtime.rs`, `src/rete/` (the native fn), and nothing else. **No call-site
churn** — `insert` keeps its name, arity and signature, so every existing caller is untouched. No
corpus migration, no codemod.

## STOP triggers (each is a rejection: ship nothing for it, report the gap)

1. **STOP-1** — if the rule-RHS form `(:wat::rete::insert <record>)` (the 1-arg form inside a
   `defrule` `:then`, interpreted by the matcher) is affected in any way by the new dispatch arm,
   STOP and report. It is a different construct from the 2-arg function and must stay that way.

2. **STOP-2** — if `facts` cannot be resolved by name through `RecordDef.field_names` for a
   `Session`, STOP and report what the lookup returned. Do not fall back to a positional index.

3. **STOP-3** — if any existing rete differential goes red, STOP and report the test name and the
   diff. The oracle is the anchor; a green new gate beside a red existing differential is not the work.

## Definition of done

- `cargo nextest run --release -E 'test(/native_insert|delegates_to_the_prime|stages_like_the_oracle|content_matches_the_oracle/)'` — all three pass.
- `cargo nextest run --release -E 'binary_id(wat::rete)'` — all pass.
- `cargo clippy --all-targets --release` — silent.
- Report `git diff --stat`.

Leave the tree dirty and uncommitted; the orchestrator weighs by its own re-run and commits.

## A prior result to copy for shape

The `fire-rules` trio is the pattern, end to end: oracle in wat, prime in Rust, public delegate in
wat, one dispatch arm. You are adding the second member of a family that already has one.
