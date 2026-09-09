# BRIEF — STONE ④: the corpus's blanket-dependents

## The work, in one paragraph

Four `.wat` files under `wat-scripts/` type-check ONLY because the `:wat::*` blanket accepts any
reserved-prefix head. The loader gate (`tests/lint/wat_scripts_fixes_load.rs:36`) requires every
`.wat` beneath `wat-scripts/` to `startup_from_source` cleanly, so all four turn it RED the moment
the blanket dies. **Two carry calls to verbs that do not exist — fix those. Two are deliberate
witnesses — move them out of the gate's reach and give them the assertion they never had.**

## Read first

`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-4-the-corpus-blanket-dependents.md` — every
disposition below is measured there. **Do not re-derive them; transcribe and verify.**

## ① `wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat:16` — FIX

```
FROM  (:wat::kernel::panic! "form-of: malformed source")
TO    (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))
```

`:wat::kernel::panic!` is implemented NOWHERE. `raise!` is not a drop-in — it expects
`:wat::core::Error`, so a string is a `TypeMismatch`. The replacement is `wat/deporder.wat:180`'s
idiom for the IDENTICAL situation (a `ReadOutcome::Malformed` arm in a form-reading fn), it
type-checks here at exit 0 (measured), and it uses the `__cause` the site currently binds and
throws away.

⚠ **This file ALREADY fails at RUNTIME (exit 1) at line 24**, on arc 109's angle-bracket wall —
`(:wat::core::keyword-node ":wat::kernel::Peer<A,B>")`. Measured BEFORE any edit. **Do not fix
that, do not delete the file, and do not report it as a regression.** Its disposition is the
orchestrator's; your job is `--check` clean, which is what the gate measures.

## ② `wat-scripts/fmt/fixtures/cond-overflow.wat:8,9` — FIX

```
FROM  :wat::string::=      (registered nowhere)
TO    :wat::core::=        (both occurrences)
```

⚠ The fixture's header is *"Nested cond whose ALIGNED width exceeds 120"* and the names differ by
two characters, so this edit could in principle drop it under the threshold and leave a fixture
that silently tests nothing. **Measured both ways through `wat-scripts/fmt/run-all.wat`:
byte-identical, both stay UNALIGNED.** Re-confirm it with the acceptance command below; the fixture
has no golden and no automated consumer, so that run is the only guard.

## ③ + ④ THE TWO WITNESSES — MOVE, then ASSERT

Both must leave `wat-scripts/` (the gate's scan root) and land under `tests/resolve/` as fixtures
with an `.rs` row pinning **today's** behaviour. They are correct files whose containment premise
is the blanket itself.

```
probe-f64-comparator-bogus-head.wat        → tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat
keyword-accessor-vs-enum-map-ctor.wat      → tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__dot_spelling_reaches_the_accessor.wat
```

Write ONE new `tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head.rs` with a row per
fixture, following `tests/resolve/probe_arc255_the_type_position_has_its_own_authority.rs` exactly
for shape (its `rel()`/`run_check()` helpers run the binary from `CARGO_MANIFEST_DIR` with a
RELATIVE path so any span in a golden is machine-independent — copy that).

**Assert today's measured behaviour:**

```
bogus_rete_head            --check exit 0   ← the defect witnessed: a bogus :wat::* head type-checks
                           run     exit≠0, raising UnknownFunction naming :wat::rete::f64::>X
dot_spelling               --check exit 0
                           run     exit 0, and its five printlns produce today's output
```

★★ **These rows are RATCHETS AIMED AT THE BLANKET, and that is the point.** When the blanket dies,
`--check` starts exiting 1 and both go RED — at exactly the right moment, in tests whose names say
why. The blanket's own stone updates them, and that update is its proof. **Say so in the file
header**, or the next person reads a red as a rot.

⚠ Preserve each fixture's original header comments verbatim — they carry the measurements that
explain WHY each file exists, including `keyword-accessor`'s sentence naming the blanket as the
mechanism. Add a line recording the move and its reason; delete nothing.

## Blast radius

Two `.wat` edits in place · two `.wat` files moved (`git mv`) · one new `.rs`. **No `src/` change.**
No change to the loader gate itself.

## Acceptance

```
target/release/wat --check wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat        exit 0
target/release/wat --check wat-scripts/fmt/fixtures/cond-overflow.wat                      exit 0
target/release/wat wat-scripts/fmt/run-all.wat wat-scripts/fmt/fixtures/cond-overflow.wat
        → the cond clauses stay UNALIGNED, exactly as before the edit
cargo nextest run --release -E 'test(probe_arc255_the_blanket_hides_a_phantom_head)'        all pass
```

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If the `cond-overflow` clauses START ALIGNING after the edit — STOP with both formatted
outputs. The fixture's purpose is the >120 threshold, and alignment means the edit voided it.

**STOP-2.** If you find yourself editing anything under `src/` — STOP. This stone is corpus-only.

**STOP-3.** If you find yourself "fixing" the angle-bracket runtime failure in ①, or deleting that
file — STOP. It is pre-existing, measured, and the orchestrator's to rule on.

**STOP-4.** If either witness's assertion needs `--check` to exit 1 to pass — STOP. That is the
POST-blanket world; these rows pin TODAY, so they can fail loudly when it changes.

**STOP-5.** If moving a fixture breaks another test that referenced its old path — STOP and report
the referrer.

## Tier

You edit and report. Run the four acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a
quiescent tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
