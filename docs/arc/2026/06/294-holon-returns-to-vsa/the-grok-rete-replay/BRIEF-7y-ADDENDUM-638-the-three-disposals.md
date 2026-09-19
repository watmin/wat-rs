# BRIEF 7y ADDENDUM — #638's three out-of-scope compile failures: the ruling

**Read `BRIEF-7y-replay-batch-4y.md` and `BRIEF-7y-ADDENDUM-638-three-genuine-out-of-scope-compile-failures.md`
first.** The STOP was correct: the gate's contract forbids runes and forbids deleting a rule to buy
green, and three files failed that grok's own disposition table never named. **Deciding their fate was
the orchestrator's call, and here it is.** Both rulings are 4-YES and rest on precedent already in this
tree — neither invents a policy.

## The three, and what they actually are

| file | origin | why it cannot compile HERE |
|---|---|---|
| `wat-scripts/scratch-pad/experiri-then/then-match-paren-arm.wat` | grok #537, still at grok's tip | `match` in a `:then`, which **this tree's fence refuses** |
| `…/then-match-bare-arm.wat` | grok #537, still at grok's tip | same |
| `wat-scripts/scratch-pad/277-width-fixpoint-probe.wat` | **main-only** (`a2ff725ad`) | its rule set is **deliberately non-stratifiable** |

## RULING 1 — the two `then-match-*` probes are DELETED (4-YES)

**Obvious.** They probe `match` inside a `:then`. This tree refuses that form by
`250162a0e` (arc 277, ACCEPTED, an ancestor of this whole replay), and **the builder ruled it 4-YES at
#324** — option A: our fence stands, grok's opposing tests were inverted to assert refusal, and
`wat/rete/compile.wat` was left untouched. These two cannot compile here **by construction**, now or
ever. ⛔ **And grok agrees about the family: grok's OWN #638 deletes two siblings from this exact
directory** — `then-law-a-core-head.wat` and `then-law-a-core-not.wat`. Deleting ours is grok's own
disposition applied consistently, not an invention.

**Simple.** Two deletions, inside the step that lands the gate.

**Honest.** Nothing is lost. The finding these probes carry — *the fence refuses a `match` in a `:then`*
— is **already pinned by main's own live test**, `tests/rete/probe_then_match_is_refused.wat`
(referenced at `probe_then_fence_and_enum_name.rs:54`). The deletion is recorded here, in #638's body,
and in the SCORE, citing #324.

**Good UX.** The gate's zero-exemption contract is honored without a rune and without weakening
anything; a future hand finds the refusal documented in a test that actually asserts it.

⛔ **Rejected:** inverting them to assert refusal (the #324 move) — these are scratch `.wat`, not tests;
they have no assertion harness, and an inverted probe still would not COMPILE, which is what the gate
demands. Rejected: a rune — the gate's DESIGN forbids one, by name.

## RULING 2 — `277-width-fixpoint-probe.wat` is RELOCATED, NOT deleted and NOT repaired (4-YES)

⛔ **READ THIS BEFORE TOUCHING IT: the probe already did its job, and its refusal IS the result.**
`docs/arc/2026/06/277-wat-lint-fix-fmt/NOTE-width-is-a-fact-not-a-rule.md` (2026-09-05) records the
harvest in its own title — *"rete CANNOT derive width. The DESIGN's mechanism is wrong, and the repair
is small."* — and cites this exact path as the evidence:

> `wat-scripts/scratch-pad/277-width-fixpoint-probe.wat`, run on one file:
> `stratify: negation cycle detected — rule set is not stratifiable`

The interior rule aggregates `Width` **over `Width`**, the relation it derives. Stratified evaluation
forbids that and this engine implements it. **The non-compilation is the disconfirmation**, isolated and
written up: the NOTE records that the identical rule aggregating over `Node` instead runs and prints.

**Obvious.** `wat-scripts/` is for loadable **working** references, and #638's gate now says every rule
there must compile. A **deliberately non-compiling, already-harvested negative result** does not belong
in that scope. It is not a script; it is evidence attached to a design NOTE.

**Simple.** One `git mv` plus one citation update in the NOTE.

**Honest.** Nothing is deleted, nothing is repaired into silence, and the NOTE keeps a path that
resolves. ⛔ **Repairing it to compile would destroy the very thing it demonstrates** — that is the
"delete a rule to get green" move the gate's DESIGN rejects, wearing a different hat.

**Good UX.** The gate stays exemption-free; the probe lands beside the note that harvested it.

### How to land ruling 2

1. `git mv wat-scripts/scratch-pad/277-width-fixpoint-probe.wat
   docs/arc/2026/06/277-wat-lint-fix-fmt/probes/277-width-fixpoint-probe.wat`
   (**precedent:** batch 4m #385 put deliberately-red `.wat` probes under `docs/arc/.../probes/`.)
2. **Measure what the docs-side gate wants.** `every_docs_wat_loads_or_declares_why_not` may require a
   `rune:lint(red-by-design)` with a reason. **Run it and read its verdict** — if it demands a rune,
   give one whose reason states: the rule set is deliberately non-stratifiable, the disconfirmation is
   recorded in `NOTE-width-is-a-fact-not-a-rule.md`, and repairing it would erase the finding.
   ⚠ This is NOT the rete-compile gate's forbidden rune — different gate, and that one HAS a declared
   escape. If the docs gate needs nothing, add nothing.
3. **Update the NOTE's citation** to the new path, and add one line: moved by replay #638 because the
   new compile gate scopes `wat-scripts/` to rules that compile; the probe's refusal is its result.
4. Re-run both `wat-scripts/` loader gates and the docs-wat gate; quote every verdict.

## Then finish the batch

Resume at **#638** with the stashed work, land **#639** and **#640**, and write
`SCORE-7y-replay-batch-4y.md` for the full 20. **The SCORE must carry both rulings**, each with the
precedent it rests on, under E4 and E13.

⛔ **If landing either ruling reveals something that contradicts it — a live consumer of a deleted probe,
a docs gate that refuses the relocation — STOP and report.** A ruling made on measurement is still a
ruling made without the tree in front of it.

## Rows this affects

`EXPECTATIONS-7y` is **not amended** (finding 34). **E4** is satisfied by the gate reaching green with
**zero runes on it** and the three dispositions recorded. **E11**'s prediction stands at **5893 run / 22
skipped** — the deletions remove no `#[test]`, and the relocation moves a file the test count never saw.
If your own count disagrees, report it.
