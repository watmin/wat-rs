# BRIEF — STONE: the collection readers get homes

Home and rule four collection verbs — `:wat::core::assoc`, `conj`, `drop`, `take`. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-collection-readers.md`.

⚠ **Four, not six.** `find-last-index` and `seqable->stream` are W7 (they run caller code) and are
out of scope — the DESIGN says why.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the scope test, and the `drop`/`take` subtlety.
2. `src/intrinsic/collection.rs` — **the template**: seven verbs homed as thin delegates, full
   directive blocks, grounding prose per axis. Its header also carries the 1-arity
   `std::slice::from_ref` idiom. These four belong here.
3. The four implementations: `eval_assoc` and `eval_conj` (`src/runtime.rs`), `eval_vec_drop` and
   `eval_vec_take` (`src/collection/transform.rs`).
4. `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`.

## The work

### 1 — home the four

One thin `#[wat_intrinsic]` delegate each, over the existing named fn. **Bodies do not move.**
Declare the real arity so each hand-rolled `args.len() != N` guard retires; remove the four literal
dispatch arms.

### 2 — rule each verb SEPARATELY

All four are expected `Pure ∧ Deterministic` — none applies caller code (verified in pre-flight).

⚠ **`@Totality` is per-verb and you measure it.** Do not copy one across the four. `assoc`/`conj`
reach container-capability gates (`StreamContainer`/`MapContainer`) that `drop`/`take` do not, so
their failure surfaces differ. Cite the line you ruled each from.

⚠ **`drop`/`take` return a lazily-constructed `Stream`.** Constructing a thunk is not running one —
that is what keeps them in scope while `seqable->stream` is not. Say so in the grounding prose, or a
later reader will re-litigate it.

### 3 — the ratchet and the ledger

Delete the four `KNOWN_UNREVIEWED` rows (45 → 41).

**A MIXED debt prediction, and it is falsifiable in both directions:** `assoc` and `conj` DO carry an
`env.register()` TypeScheme, `drop` and `take` do NOT. So expect
`FROZEN_CHECKER_DEBT_LEDGER` 62 → 64 — **rows for `drop` and `take` only**.

### 4 — the probe

`wat-scripts/scratch-pad/255-probe-the-collection-readers.wat`, following the shape of the others:
the four still behave as today, on a vector and (where accepted) a stream.

## Blast radius

`src/intrinsic/collection.rs` · `src/runtime.rs` (four arms out; `eval_assoc`/`eval_conj` become
`pub(crate)`) · `src/collection/transform.rs` (visibility only) · `src/rete/purity.rs` (four rows
out) · `src/intrinsic/mod.rs` (two ledger rows) · the new probe. No body moves. No `.wat` change.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — the two W7 verbs are not in scope.** If homing the four seems to require touching
`find-last-index` or `seqable->stream`, STOP and report. Both run caller code; ruling either `Pure`
would be falsifiable in one line.

**STOP-2 — the debt prediction is falsifiable in BOTH directions.** If `assoc`/`conj` turn out to
need a row, or `drop`/`take` turn out not to, STOP and report which — a uniform outcome means the
measurement was wrong in a way a uniform guess would have hidden.

**STOP-3 — do not copy a `@Totality` across the four.** Each is measured from its own body with a
cited line. If two genuinely share a verdict, say so and cite both.

**STOP-4 — no body moves.** If any of the four cannot be a thin delegate over its existing named fn,
STOP and report what forced it.

## Report

Per-file diff summary; the four rulings **each with the line you read**; whether the mixed debt
prediction held in both directions; the probe's output from the pre-existing binary. Then the part
the orchestrator cannot reconstruct: what surprised you — a body that reaches a failure surface the
DESIGN did not name, a verb that turned out to run caller code after all, or an arity that differed.
