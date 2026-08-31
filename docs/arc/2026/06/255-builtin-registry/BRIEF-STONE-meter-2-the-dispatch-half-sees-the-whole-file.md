# BRIEF — STONE meter-2: the completeness gate's dispatch half sees the whole file

Make `dispatch_verbs` (`src/rete/purity.rs`) find dispatch arms by **shape**, anywhere in the file,
instead of by scanning between two named function anchors. Then dispose every verb that newly
becomes visible. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-meter-2-the-dispatch-half-sees-the-whole-file.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned contract decision (population defined by SHAPE, not by enclosing
   function name) and the two dispositions available for a scream.
2. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-meter-1-the-scan-walks-the-tree.md` — **the
   precedent.** It fixed the *registration* half of this same gate, predicted "~25 verbs will
   scream", measured **eleven**, and disposed them with named `KNOWN_UNREVIEWED` rows. Copy its shape
   and its honesty about the prediction.
3. `src/rete/purity.rs`, `fn dispatch_verbs` — the two-anchor loop, and `walk_intrinsic_homes` beside
   it (meter-1's fix, for the shape of a whole-scope scan).
4. `src/rete/purity.rs`, `KNOWN_UNREVIEWED`'s doc comment — the ratchet's rules, and what it says
   about the list being a last resort.
5. `src/runtime.rs`, `fn eval_list` around the `:wat::core::Some` arm — the second arm shape.

## The work

### 1 — scan by shape, whole file

Replace the two-anchor span scan with one that reads the whole file and recognises **both** arm
shapes keyed on a wat FQDN:

```rust
    ":wat::core::x" => …                                  // literal arm
    WatAST::Keyword(k, _) if k == ":wat::core::x" => …    // keyword-guard arm
```

⚠ **Validate your pattern before trusting its count.** A text scan over a whole 33k-line file will
pick up things a span scan never reached — a FQDN inside a comment, a doc example, a message string,
a `matches!` that is not a dispatch arm. Read a sample of what your pattern collects and say in your
report what you excluded and why. A count you have not validated is a guess.

### 2 — measure what actually screams

With the scan widened, the completeness gate's population grows and its ratchet fires for every
newly-visible verb with no ruling. **Report the real list.** The DESIGN predicts roughly 38 from a
text scan; meter-1 predicted ~25 and measured 11. Your measured number is the answer, whatever it is.

### 3 — dispose each scream, one at a time

Per verb, choose **one**:

- **RULE IT** — where the implementation makes the answer plain. Cite the line you read.
- **A NAMED `KNOWN_UNREVIEWED` ROW** — where the ruling is genuinely open. The row must carry a
  reason saying *why* it is open. A row without a reason is the laundering the gate exists to
  prevent.

These verbs have been dispatched all along; a row records **pre-existing debt now visible**, not new
debt waved through. Say so in the rows you add.

Expect `:wat::core::def`, `:wat::core::defalias`, `:wat::core::fn` among the screams — the
declaration door, a known open question. A `KNOWN_UNREVIEWED` row naming that question is the right
disposition; do not try to rule them.

`:None` and `:undefined` may not be verbs at all. If a scream is not a verb, say so and dispose it by
excluding it from the scan's shape with a stated reason — not by adding a row that pretends it is one.

## Blast radius

`src/rete/purity.rs`. No changes to `src/runtime.rs`'s dispatch, to any verb's evaluation, or to any
registration. No homing in this stone.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — do not add function-name anchors.** If shape-based scanning proves hard, STOP and report
what blocked it. Adding the six missing names to the anchor list is explicitly disqualified: the
defect is anchoring on names, and `dispatch_keyword_head` sits one word from the anchored
`dispatch_keyword_head_value`.

**STOP-2 — no ruling you cannot cite.** If a verb's ruling is not plain from its implementation, it
gets a `KNOWN_UNREVIEWED` row with a reason — never a guessed `Pure`/`Total` to keep the count down.

**STOP-3 — a row must say why.** If you cannot state why a verb's ruling is open, you have not
looked at it yet. STOP and report it rather than adding a bare row.

**STOP-4 — if the screams exceed what one stone can dispose.** If the measured list is large enough
that disposing it honestly would be a campaign rather than a stone, STOP after reporting the full
measured list and dispose nothing. The measurement is the deliverable in that case, and the
orchestrator re-plans.

## Report

Per-file diff summary; **the pattern you scanned with and what you excluded from it, with reasons**;
the full measured list of screams; and each verb's disposition with its citation. Then the part the
orchestrator cannot reconstruct: what surprised you — an arm shape neither the design nor this brief
names, a verb that turned out not to be one, or a place where the scan's honest population was
harder to define than "keyed on a wat FQDN".
