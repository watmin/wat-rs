# BRIEF 4 — ADDENDUM: a latent branch defect that main's walls surface

Batch 1 stopped at #11 (`SCORE-4-replay-batch-1.md`, STOP-2). The STOP was right; its attribution
("the chain missed a migration") was not, and this addendum is the rule for the class. Resume at #11
from the staged overlay you left.

## What #11 is (measured by the orchestrator, 2026-09-13)

- `:wat::core::edn::to-string` **never existed** — not at the merge base `de827fb4c`, not at #11
  `eebf75374` (whole-tree search: its only occurrence is `rete-differential.wat:134` itself). Its
  siblings (`:wat::core::i64::to-string`) are named literally in `src/`, so no family generated it.
- It resolved on grok-rete because the resolver waved EVERY `:wat::*` head through
  (`if is_reserved_prefix(head) { return true; }`). Main's `c3fefc5ab` (STONE 255 F+G, "THE :wat::*
  BLANKET IS DEAD") made resolution registry-only, so main refuses it at startup. The call sits on the
  fuzzer's MISMATCH-report branch, which never ran on grok-rete — a latent defect, not a rete change.
- Grok-rete's author fixed it at #19 `a39c28e10`; that text prints `d`/`f`, which do not exist at #11,
  so it cannot be carried back. No recorded migration applies: nothing was ever renamed.
- Main's registered verb for "a value's EDN text" is `:wat::edn::write` (`@arg v :T`, `@ret
  :wat::core::String`, "the compact single-line EDN text", `src/intrinsic/edn.rs:152-156`).

## The rule — RULED 2026-09-13 (four questions, 4 YES)

A `:wat::*` call head that exists on NEITHER side (grok-rete's tree at C, main's registry), which main's
registry-only resolver now refuses at startup:
1. **Re-express that one call head** with main's registered verb of the same meaning, at that one site.
   At #11: `(:wat::core::edn::to-string c)` → `(:wat::edn::write c)`.
2. **Log it** in the step's row as `LATENT (c3fefc5ab)`, with both spellings, the file:line, and the
   registered verb's `@ret`.
3. **When the author's own later fix arrives** (here #19), its merge conflict on that hunk resolves to
   the author's text (C), and the log says so.

This is a one-site defect repair, not a corpus migration (R21 governs migrations). STOP-2 still fires for
a CHAIN defect (a codemod missed or broke something).

**STOP-7 (new):** no registered verb has the call's meaning, or the repair needs more than the call head
(a changed argument, a new binding). Report the site and the evidence; do not invent a fix.

## Resume

From the staged #11 overlay: apply the rule at `wat-scripts/fuzz/rete-differential.wat:134`, re-run the
step's gate (`--check` both produced `.wat`, `rete_fuzzer_finds_no_native_oracle_divergence` by name),
commit `REPLAY(grok-rete #11)`, and continue to #60 under BRIEF-4. #17 also touches this file; #19 carries
the author's fix. Yield after #60, or at the first STOP, updating `SCORE-4-replay-batch-1.md` and
`REPLAY-LOG.md`.
