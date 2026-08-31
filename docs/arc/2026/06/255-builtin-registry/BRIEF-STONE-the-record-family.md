# BRIEF — STONE: the record family gets homes — ALL SEVEN

Home and rule the seven aggregate verbs. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-record-family.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `target/release/wat --check <file>` for a fast read, remembering it does
not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — especially **why the struct pair is in scope**: `struct-field` is not a struct
   verb, arc 293.R2.2 deleted its `Nature::Struct` guard, and it is the unified field-read every
   record/holon-record/struct accessor calls.
2. `wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat` — the committed evidence.
   Its header carries the out-of-tree fence runs verbatim.
3. `src/intrinsic/record.rs` (65 lines) — **the template and the home.** `Record/field-at` is
   already a thin delegate here with a full directive block. All seven join it.
4. `src/intrinsic/collection.rs` header — the **1-arity delegate idiom**
   (`std::slice::from_ref(v)`, never `&[v.clone()]`). `record->map` is 1-arity; clippy caught this
   on the last two stones.
5. The seven implementations, all named fns in `src/runtime.rs`, reached from the dispatch arms at
   `:5548` `:5562` `:5573` `:5578` `:5830` `:5831` `:5832`.
6. `src/check.rs:21236` (`record->map`), `:21259` (`Record/assoc`), `:21275` (`Record/same-data?`)
   — the three registered `TypeScheme`s. **Read these before writing any `@arg`/`@ret`** (§4 below).

## The work

### 1 — home the seven in `src/intrinsic/record.rs`

One thin `#[wat_intrinsic]` delegate each, calling straight into the existing named fn. **Bodies do
not move.** Remove their seven literal dispatch arms; the seven fns become `pub(crate)`.

Update the module header: it is no longer "the first `Record::*` accessor verb" — it is the
**aggregate family** (construction, field read, conversion, and the enum-variant constructor).

⛔ **THE ARITIES ARE NOT UNIFORM AND TWO ARE VARIADIC — measured from each fn's first guard:**

```
:wat::core::to-record          args.len() != 2      fixed 2
:wat::core::record->map        args.len() != 1      fixed 1
:wat::core::Record/assoc       args.len() != 3      fixed 3
:wat::core::Record/same-data?  args.len() != 2      fixed 2
:wat::core::struct-field       args.len() != 2      fixed 2
:wat::core::variant            args.len()  < 2      ⛔ VARIADIC, minimum 2
:wat::core::struct-new         args.len()  < 2      ⛔ VARIADIC, minimum 2
```

A fixed arity declared on either variadic verb breaks every call with more fields — a 3-field
`(struct-new :T a b c)` is arity 4. Declare them the way the registry expresses a variadic verb;
**find that spelling from an existing variadic registration, do not invent one.**

### 2 — the rulings

**All seven: `@Purity Pure`, `@Determinism Deterministic`.** The grounding prose per verb must say
why *from its body*, not by copying a neighbour. For the struct pair, the prose cites the measured
fact: a field read is a projection — same input, same answer — and arc 293.R2.2's own comment says
the `Nature::Struct` guard was deleted as a pre-unification artifact.

⚠ **`@Totality` is YOURS to measure, per verb, cited by line.** Do not copy one across the seven.
The collection readers proved why: `assoc`/`conj` were `Partial` on inner helpers that a
container-gate reading never reaches. **`Record/assoc` is `assoc`'s sibling — read the inner
helpers.** `struct-field` has a field-index bounds check and a non-`Aggregate` receiver
`TypeMismatch`; decide what each means for this axis and cite the line you read.

### 3 — satisfy the ratchet

Delete the seven `KNOWN_UNREVIEWED` rows in `src/rete/purity.rs` — lines `2247` `2248` `2264`
`2268` `2269` `2271` `2273`. **41 → 34.**

### 4 — the doc gate, and how to pass it FIRST TIME

`doc_arg_ret_types_match_checker_scheme` verifies `@arg`/`@ret` against the registered scheme. It
went RED twice on the collection readers — `conj`'s `@arg` was *narrower* than its scheme,
`assoc`'s was *more precise*. **A verb WITH a scheme is a verb the gate checks.**

For the three at `check.rs:21236/21259/21275`, read the `TypeScheme` and write `@arg`/`@ret` to
match it **exactly**; put the real meaning in the prose instead. `record->map`'s params are
`[record_ty()]` and its ret is a parametric `wat::core::HashMap` of `[keyword, T]` — mirror the
scheme, do not improve on it.

### 5 — the predicted debt rows

Measured against `register_builtins` (`src/check.rs:16569–21711`):

```
Record/assoc · Record/same-data? · record->map    scheme YES  ->  NO debt row
to-record · variant · struct-new · struct-field   scheme NO   ->  a row each
```

Expect `checker_skip_debt_is_named_and_frozen` to demand four `FROZEN_CHECKER_DEBT_LEDGER` rows,
each with a reason. **64 → 68.**

### 6 — the probe

`wat-scripts/scratch-pad/255-the-record-family-homed.wat`, following the shape of the others there:
the seven still behave as before, and `metadata-of` shows each one's declared axes.

## Blast radius

`src/intrinsic/record.rs` (seven delegates + header) · `src/runtime.rs` (seven arms out; seven fns
become `pub(crate)`) · `src/rete/purity.rs` (seven ledger rows out) · `src/intrinsic/mod.rs` (four
debt rows) · the new probe. No body moves. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**STOP-1 — the arity table is measured, not guessed.** If any fn's real first guard disagrees with
§1, STOP and report which and what you read. An arity declared wrong retires a guard that was
protecting something.

**STOP-2 — do not rule `@Totality` by family symmetry.** Seven verbs, seven measurements. If two
genuinely share a verdict, each still needs its own cited line.

**STOP-3 — the debt prediction is falsifiable in both directions.** If any of the four DOES carry an
`env.register()` scheme, or any of the three does NOT, STOP and report which — the measurement is
wrong, and an unearned ledger row is the laundering the ledger's own doc forbids.

**STOP-4 — no body moves.** If any of the seven cannot be a thin delegate over its existing named
fn, STOP and report what forced it.

**STOP-5 — you are not re-ruling `accessor_meta`.** `src/rete/purity.rs`'s
`pure: a.nature.is_pure()` stays exactly as it is. This stone does not touch arc 293.W's wall, and
`effectful_by_prefix` is not edited for any reason.

## Report

Per-file diff summary; the seven `@Totality` rulings **with the line you read for each**; whether
the arity table held; whether the debt prediction held; and the probe's output from the pre-existing
binary. Then the part the orchestrator cannot reconstruct: **what surprised you** — a body that did
not match its sibling, a guard the brief did not name, an arity that was not what §1 said.
