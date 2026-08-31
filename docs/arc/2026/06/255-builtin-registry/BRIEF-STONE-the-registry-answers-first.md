# BRIEF — STONE: the registry answers FIRST — retire the three prefix guesses

Move eleven stranded totality facts into the registry, then delete the three prefix guesses that
were carrying them. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-registry-answers-first.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `target/release/wat --check <file>` for a fast read, remembering it does
not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the seventeen verdicts that outrank the registry, and why these three go first.
2. `src/rete/purity.rs:246` — `intrinsic_meta`. Read it from `:246` to the registry lookup at
   `:578` so you can see the whole ordering before you cut anything out of it.
3. `src/rete/purity.rs:296–317` — the `:wat::string::` / `:wat::regex::` guess **and its comment**.
   The comment ARGUES the totality of the eight; it is the reasoning you are relocating.
4. `src/rete/purity.rs:318–338` — the `:wat::edn::` guess, same shape, three names.
5. `wat/runtime-meta.wat`, the `Totality` `defenum` — what `Total` / `Partial` / `Unreviewed` each
   MEAN. You are writing one of these per verb and the definitions are the bar.

## The work

### 1 — move the eleven facts IN

Each verb below already carries `@Totality Unreviewed` at its registration. Replace that with the
measured verdict, and add grounding prose saying **why, from the body**:

```
src/intrinsic/string.rs:261   :wat::string::length          :129   :wat::string::contains?
src/intrinsic/string.rs:289   :wat::string::trim            :161   :wat::string::starts-with?
src/intrinsic/string.rs:320   :wat::string::to-lowercase    :193   :wat::string::ends-with?
src/intrinsic/string.rs:705   :wat::string::concat          :231   :wat::string::empty?
src/intrinsic/edn.rs:120      :wat::edn::read-foreign
src/intrinsic/edn.rs:282      :wat::edn::ForeignRecord/get  :307   :wat::edn::ForeignRecord/class
```

⛔ **RE-DERIVE EACH FROM ITS BODY. Do not transcribe the guess's comment.** The comment tells you
what a prior self concluded and is a good place to check yourself against — it is not evidence. A
verdict copied rather than re-read is a hand-list that changed address, which is the exact shape this
stone exists to remove. Cite the line you read for each of the eleven.

### 2 — delete the three prefix guesses

Remove the `head.starts_with(":wat::string::") || head.starts_with(":wat::regex::")` block and the
`head.starts_with(":wat::edn::")` block from `intrinsic_meta` entirely, including their nested
`matches!` hand-lists. The registry lookup below them then answers for all 34.

Leave a short comment at the cut naming what retired and why (the registry now carries the facts),
in the shape the arc's other retirements use.

### 3 — what must NOT change

The 34 shadowed verbs all declare `@Purity Pure` and `@Determinism Deterministic`, which is exactly
what the guess returned. **Do not edit a single `@Purity` or `@Determinism` line in this stone.**
The 23 verbs that were NOT in the hand-lists stay `@Totality Unreviewed` — untouched, honestly
unmeasured. Measuring them is a different stone.

### 4 — the probe

`wat-scripts/scratch-pad/255-the-registry-answers-first.wat`. The load-bearing demonstration is
**behaviour preservation** through `:wat::rete::compile-all`:

- a `where` using `(:wat::string::concat …)` — **ADMITTED**, exactly as before this stone
- a `where` using a string verb that stays `Unreviewed` (e.g. `:wat::string::split`) —
  **REFUSED**, exactly as before

Follow the shape of `wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat`,
including its header convention: a committed `.wat` must LOAD, so anything that panics at run is
demonstrated out-of-tree and recorded verbatim in the header.

## Blast radius

`src/rete/purity.rs` (two blocks deleted) · `src/intrinsic/string.rs` (eight `@Totality` lines) ·
`src/intrinsic/edn.rs` (three `@Totality` lines) · the new probe. No body moves. No `.wat` corpus
change. No new registrations.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**STOP-1 — two unregistered heads live under these prefixes.** `:wat::string::=` and
`:wat::string::not=` are rete-IR spellings (`src/rete/expr_ir.rs:1254–1255`) with **no**
`#[wat_intrinsic]` registration and zero corpus usage. `rete_op_index` keys only `rete_name`
(`src/rete/vocabulary.rs:1437`), so the CORE-spelled forms fall into the guess today and get
`pure ∧ deterministic` from it. After the cut they get no verdict at all. **Measure whether anything
asks `intrinsic_meta` about either in core spelling.** If something does, STOP and report it — the
stone needs re-scoping, and inventing a carve-out to keep them alive is not yours to decide.

**STOP-2 — a verdict you cannot re-derive is a STOP.** If reading a body does not let you conclude
`Total` or `Partial` for one of the eleven, STOP and report which and what blocked you. Leaving it
`Unreviewed` would silently narrow the fence; guessing `Total` to preserve behaviour is the lie
`Unreviewed` exists to prevent. **Neither is available to you.**

**STOP-3 — the eleven are a measured population, not a starting point.** If you find a twelfth verb
under these prefixes whose behaviour would change, STOP and report it. The DESIGN's claim is that
exactly eleven facts are stranded; a twelfth means the measurement was wrong.

**STOP-4 — no `@Purity` or `@Determinism` edits.** If any of the 34 looks mis-declared on those
axes, that is a finding to report, not a line to change here.

**STOP-5 — `rete_op_for`, the singletons, and the three `matches!` sets stay exactly as they are.**
Fourteen other verdicts still outrank the registry. They are later waves. Touching one makes a red
un-attributable.

## Report

Per-file diff summary; the eleven `@Totality` verdicts **each with the line you read for it**;
whether STOP-1's measurement found any core-spelled consumer; the probe's output from the
pre-existing binary; and the part the orchestrator cannot reconstruct: **what surprised you** — a
body that did not support the comment's claim, a verb whose totality is not what a prior self
argued, or a consumer the brief did not name.
