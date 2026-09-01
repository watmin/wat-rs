# BRIEF — STONE: wave 3 — the last five guards become registrations

Home five verbs, delete their five guards, leave the four `verify::` locator tags alone. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-registry-answers-first-wave-3.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` freely; that binary does NOT
contain your Rust changes, which makes it the right tool for capturing BEFORE behaviour.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything.

## Read in order

1. The DESIGN above — the 5/4 split, and why the `verify::` four are not verbs.
2. `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`
   — the rule you rule `macro-error` by.
3. `src/intrinsic/option.rs` and `src/intrinsic/result.rs` — the delegate template, and the
   precedent that matters here: `Option/try`/`Result/try` were ruled **Total** on a propagation
   SIGNAL while their `expect` sibling was ruled **Partial** on a raise. Opposite verdicts, same
   family.
4. `src/runtime.rs:5533` `:5538` (`aggregate-new`, `kwargs-construct`) · `:5371` `:5376`
   (`write-forms`, `with-children` → `crate::edn::render::eval_*`) · `:5390` (`macro-error`, the
   only inline body).
5. `src/intrinsic/collection.rs` header — the 1-arity delegate idiom (`std::slice::from_ref`, never
   `&[v.clone()]`). Clippy has caught this on three stones.

## The work

### 1 — home the five

One thin `#[wat_intrinsic]` delegate each, in the intrinsic module that fits its namespace. **Bodies
do not move**, including the two that live in `crate::edn::render`. Declare each verb's real arity,
measured from its own first guard — ⚠ **read the guard; do not infer arity from a sibling.** A
previous brief in this campaign asserted a minimum arity by pattern-matching a sibling and was wrong.

`macro-error` is the only one whose body is inline. Give it the smallest honest treatment: a named
fn it can delegate to, or a delegate carrying the body as-is — whichever leaves `runtime.rs`'s arm
deletable without moving logic.

Remove the five literal dispatch arms.

### 2 — the rulings, from the bodies

`@Purity` and `@Determinism` derived per verb and cited. `@Totality` **measured per verb** — never
copied across the five.

★ **`macro-error` is the ruling this stone exists for.** Its body always returns `Err(MacroAbort)`
and never produces a value. Decide against the RULING whether a macro-abort **signal** is a raise
(`Partial`) or a matchable propagation (`Total`), and **cite the line that decides it**. The `try`
pair is the precedent for a signal that is not a raise; `Result/expect` is the precedent for one that
is. Family resemblance decides nothing here — `try` and `expect` are siblings with opposite verdicts.

### 3 — delete the five guards, keep the sixth

Delete the `intrinsic_meta` blocks for the five (`purity.rs` — `aggregate-new`/`kwargs-construct`,
`write-forms`, `with-children`, `macro-error`), each with a retirement comment in the shape waves 1–2
used.

⛔ **The `:wat::verify::` block stays BYTE-IDENTICAL.** Those four are locator tags matched inside
`resolve_verify_payload` (`runtime.rs:24503`), not call heads.

### 4 — the predicted ledger movement

```
write-forms · with-children                      register_builtins: YES -> NO debt row
aggregate-new · kwargs-construct · macro-error   register_builtins: NO  -> a row each
FROZEN_CHECKER_DEBT  68 -> 71
KNOWN_UNREVIEWED     34 -> 34   (none of the five is on it — measured)
```

### 5 — the probe

`wat-scripts/scratch-pad/255-the-registry-answers-first-wave-3.wat`, following waves 1–2's shape:
capture BEFORE with the pre-existing binary in the header, then show each of the five still behaves
as it does today and `metadata-of` reports its declared axes. Include a control that is NOT part of
this stone and must not move, so the probe can distinguish "these five moved" from "everything
changed".

⚠ A committed `.wat` must LOAD. Anything that panics at run is demonstrated out-of-tree and recorded
verbatim in the header.

## Blast radius

`src/intrinsic/*` (five delegates) · `src/runtime.rs` (five arms out; fns become `pub(crate)`) ·
`src/rete/purity.rs` (five guard blocks out) · `src/intrinsic/mod.rs` (three debt rows) · the new
probe. No body moves. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**STOP-1 — arity is read, never inferred.** Measure each verb's own first `args.len()` guard. If one
disagrees with what a sibling's shape suggests, the guard wins. Report any that surprised you.

**STOP-2 — `macro-error` by resemblance is refused.** If the body does not let you decide between
raise and signal, STOP and report what blocked you. Do not pick the verdict that matches a
neighbour, and do not pick `Unreviewed` — the body is being read, so "nobody looked" is unavailable.

**STOP-3 — the `verify::` four are not yours.** If one looks wrong, or you find evidence that
`intrinsic_meta` really is asked about them, that is a **finding to report** — the DESIGN records the
open question deliberately. Never edit that block.

**STOP-4 — the ratchet must NOT move.** `KNOWN_UNREVIEWED` is predicted to stay at 34. If it
changes, my measurement was wrong: STOP and report which verb was on it. A shrinking ratchet looks
like success and would be a defect here.

**STOP-5 — no body moves.** If any of the five cannot be a thin delegate over its existing
implementation, STOP and report what forced it.

## Report

Per-file diff summary; the five rulings **each with the line you read**, and `macro-error`'s in full
with the RULING clause it turns on; whether the arity table held; whether the ledger predictions held
in both directions; the probe's BEFORE/AFTER; and the part the orchestrator cannot reconstruct:
**what surprised you** — a body that did not match its arm's comment, a cross-module delegate that
resisted, or evidence about the `verify::` four either way.
