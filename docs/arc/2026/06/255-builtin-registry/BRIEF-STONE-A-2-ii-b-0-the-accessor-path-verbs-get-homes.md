# BRIEF — STONE A-2-ii-b-0: the three accessor-path verbs get homes and rulings

Home and rule three verbs — `:wat::core::Option/expect`, `:wat::core::Record/field-at`,
`:wat::core::type` — so a generated record accessor stops classifying impure when reached through an
environment binding. All three are `KNOWN_UNREVIEWED` today, so they default-deny on every axis, and
that is what blocks `wat/query/mem.wat:136,163`. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-ii-b-0-the-accessor-path-verbs-get-homes.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the rulings table, and which one you must measure rather than transcribe.
2. `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`
   — the rule you are ruling by. A raise is not a matchable outcome; a raising verb is `Partial`.
3. `src/intrinsic/collection.rs` — **the template.** Seven verbs homed as thin `#[wat_intrinsic]`
   delegates over existing named fns, each with a full directive block
   (`@Purity`/`@Determinism`/`@Totality`/`@ExpandTime`/`@Category`/`@arg`/`@ret`/`@example`) and prose
   grounding each axis. Copy its shape, including the grounding prose.
4. `src/intrinsic/i64.rs` around `:wat::i64::/` — **the only `@Totality Partial` in the tree today.**
   Read how it words its partiality; yours should read like it.
5. The three implementations, all already named fns — `eval_option_expect`, `eval_record_field_at`,
   `eval_type` (all in `src/runtime.rs`, reached from thin literal arms at `:5712`, `:5451`, `:5382`).
6. `src/rete/purity.rs`'s `KNOWN_UNREVIEWED` — read its doc comment on the ratchet before deleting
   any row.

## The work

### 1 — home the three

One `#[wat_intrinsic]` delegate each. The bodies do **not** move: all three are already named fns,
so the delegate calls straight into the existing one, exactly as `src/intrinsic/i64.rs`'s
`eval_i64_add` does. Their literal dispatch arms in `runtime.rs` come out — the registry answers
first.

Declare the real arity so the hand-rolled `args.len() != N` guards retire, as the collection wave did.

### 2 — rule the axes, from the implementation

Two are pinned by measurement; **one you must measure yourself**:

- `:wat::core::Option/expect` → Pure · Deterministic · **Partial**. It raises on `None`.
- `:wat::core::Record/field-at` → Pure · Deterministic · **Partial**. Measured at the site:
  `if index < 0 || (index as usize) >= fields.len()` returns `Err`.
- `:wat::core::type` → Pure · Deterministic · **you determine**. `eval_type` has four `return Err`
  paths. Arity failures retire on homing. If any remaining path is a **domain** failure — a value it
  cannot name a type for — it is `Partial`; if the only failures are arity, it is `Total`. **Read the
  function and say which, with the line you read.**

Write the grounding prose the template uses — a sentence per axis saying *why*, citing what you read.
A directive with no ground is the thing this arc keeps finding.

`@ExpandTime`: all three are pure ∧ deterministic and safe during expansion. A `Partial` verb can
still be expand-time-legal — `macros/eval.rs` says so for `i64::/`.

### 3 — satisfy the ratchet

Delete the three rows from `KNOWN_UNREVIEWED` in `src/rete/purity.rs`. That list is a
two-directional ratchet: a verb in it that is no longer unreviewed makes the floor RED. Deleting the
rows is part of the stone, not cleanup after it.

### 4 — the probe

Write `wat-scripts/scratch-pad/255-probe-the-accessor-classifies-pure.wat`, following the shape of
the existing probes in that directory (header comment recording what was measured and why, then
`:user::main` printing one row per line):

- a record field accessor reached **through a binding**, asked via `:wat::rete::pure?` → must print
  `true` (prints `false` today);
- an **effectful** fn through a binding → must still print `false` (no widening).

## Blast radius

`src/intrinsic/` (the three new delegates — put each in the module its namespace belongs to,
following how `collection.rs` and `i64.rs` are organised) · `src/runtime.rs` (three literal arms
out) · `src/rete/purity.rs` (three `KNOWN_UNREVIEWED` rows out) · new probe. No behaviour change to
any verb's evaluation. No changes to `sort`, `sort$native`, `src/collection/transform.rs`, or
`src/freeze.rs`.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — do not declare `Total` to avoid a `Partial` row.** The two pinned `Partial`s are the
deliverable; they are the first real entries on the totality census the `expect` purge will use. If
your reading of an implementation disagrees with the DESIGN's pinned ruling, STOP and report the
disagreement with the line you read — do not quietly ship either verdict.

**STOP-2 — `type`'s totality is a measurement, not a default.** If you cannot determine from
`eval_type` whether a non-arity failure path is reachable, STOP and report what you found. Do not
pick `Total` because it is convenient or `Unreviewed` because it is safe — `Unreviewed` for a verb
you were sent to rule is the "did not look" answer this arc exists to eliminate.

**STOP-3 — the ratchet is not to be worked around.** If deleting a `KNOWN_UNREVIEWED` row appears to
require adding a different one, or adding a name anywhere to keep a gate green, STOP and report.
Never add a line to make a red gate green.

**STOP-4 — if a home does not fit the two-layer shape.** If any of the three cannot be a thin
delegate over its existing named fn — if the body would have to move or change — STOP and report
what forced it. The bodies are not in scope.

## Report

Per-file diff summary; the three rulings **with the line you read for each**, especially `type`'s;
the probe's output as your pre-existing binary reports it (noting it lacks your Rust changes). Then
the part the orchestrator cannot reconstruct: what surprised you — an implementation that did not
match its name, a fourth verb in the path you found unruled, or a place where the ruling was harder
to call than the DESIGN implied.
