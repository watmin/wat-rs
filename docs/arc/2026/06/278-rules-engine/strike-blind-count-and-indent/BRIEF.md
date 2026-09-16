# BRIEF — C10 + C11: name the gate that is blind, and stop eating an indent

Two small, independent corrections in the census tests. Both are description-layer defects with the
diagnosis already measured; neither needs new machinery.

⚠ **These are deliberately in one brief because they are two-comment-and-one-format-string edits in
adjacent files. Do them as two separate passes and report them separately** — if either turns out to
be more than it looks, STOP and say so rather than folding them together.

## PIECE 1 — C10: the 80,200 gate cannot see the branch it implies

`src/rete/kernel/tests/accum_cost.rs:52` asserts `compiled:calls == 80_200`. That counter is a
**deliberate union**: `fire/delta.rs:78` bumps it in the `skip_span` arm and `compiled_cond.rs:928`
bumps it inside `exec_compiled_with_key_ids`, the *other* arm. The file's own doc at `:15-17` already
says so — *"occupancy leaf-fill, skip-span, and `exec_compiled_with_key_ids` all increment it"* — so
the comment is **honest**; what is missing is that a reader cannot tell the assertion is therefore
blind to which arm produced the count.

**Driven by the orchestrator (2026-09-02), and this is the whole justification:** with `skip_span`
forced to `false` in `fire/delta.rs:71`, `accum_matcher_op_census` **PASSED** while
`c4_probe_bind_only_decides_skip_span_for_the_accum_axis` **FAILED**.

```
PASS  wat rete::kernel::tests::accum_cost::accum_matcher_op_census
FAIL  wat rete::kernel::tests::accum_alpha_cost::c4_probe_bind_only_decides_skip_span_for_the_accum_axis
      assertion `left != right` failed: ... identical pools mean the two benchmark arms measure the
      same path after all
```

**The work is a cross-reference, not a new gate.** Add a sentence to the `assert_eq!(calls, 80_200)`
at `:52` recording that the count is a union of three sources, that it therefore cannot discriminate
the arm, and that the discrimination lives in
`c4_probe_bind_only_decides_skip_span_for_the_accum_axis` (`accum_alpha_cost.rs`), which reads the
bind-pool length instead. Cite the mutation above as the evidence.

**Do NOT** split the counter or add a new one — that is an engine edit on the hot path for an
instrument's benefit, and the discrimination already exists.

## PIECE 2 — C11: a `\`-newline continuation eats the row indent

In `accum_cost.rs`'s `accum_alpha_leftover_split`-style in-fire block, three rows are written indented
under their parent and print **flush-left**: Rust's `\`-newline string continuation strips the
*leading* whitespace of the continued line, so the intended indent never reaches stdout — it only
shortens the pad. Confirmed in the orchestrator's own captured output:

```
in-fire
setup:seen                       0.00 ms      <- these three were written indented
alloc                          0.00 ms
insert                         0.00 ms
```

(That block has since changed shape, so **find the live instances rather than trusting this sample** —
it is illustrative, not a citation. This defect cost a previous rider real time: it nearly "fixed" an
indentation regression that never existed, because a brief quoted a table with the indent it *should*
have had.)

Fix the rendering so an intended indent survives — move the spaces after the label, or use an
explicit escape. Keep every number and column position otherwise identical.

## Blast radius

`src/rete/kernel/tests/` only. **Nothing under `src/rete/kernel/fire/`.**

## STOP triggers

1. **If piece 1 tempts you toward a new counter or an engine edit**, stop — the cross-reference is the
   whole job.
2. **If piece 2's indent fix moves any number or column**, stop and report — this is a rendering fix,
   not a re-layout.
3. **If you find more than a handful of `\`-newline indent victims**, stop and report the list rather
   than sweeping them; a wide reformat is a different strike.
4. **If either piece turns out to be larger than described**, stop and report which.

## Mutation proof

For piece 2 only: after the fix, **remove the indent again** and confirm the output visibly changes.
There is no assertion to break here — the proof is the rendered bytes, so quote them.

Piece 1 has no mutation: it adds no gate. Its evidence is the orchestrator's engine mutation above,
already run.

## What to report

- Piece 1: the comment you added, verbatim.
- Piece 2: the block before and after, **as raw bytes** (`cat -A` or equivalent), so the indent is
  visible rather than asserted.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- Anywhere this brief was thin or wrong. Five riders running; every one has found a real defect in
  the brief, including two of my own false claims. Be blunt.

Do not commit.
