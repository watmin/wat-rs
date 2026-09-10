# DESIGN + BRIEF — ③b-ii ⑧: the residue, diagnosed not assumed

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`.

## State

The dot flip is landed on the substrate and the corpus. `c15d76e01`, unpushed, tree clean.

```
floor   5334 tests run   5187 passed   147 FAILED   22 skipped      (was 795)
```

The roster has been 5334 throughout — nothing lost, nothing added. **A red start is the progress
meter.** You are driving 147 → 0.

## ⛔ THREE RULES THIS LANDING PAID FOR

**① THE SAME LITERAL CAN MEAN TWO THINGS.** In `tests/types/`, `:usr::Shape::Circle` is a `defenum`
VARIANT in `probe_arc296_h2__variant.wat` and a `defrecord` NAME in `probe_arc296_h2__record.wat` —
that family reuses one name across type-kinds deliberately. A global token→token rename is therefore
**unsound**; it was safe across the stdlib only because names are unique there. **An ask-confirmed
rename is confirmed for the file it was derived from and nowhere else.**

**② THE FAILING-TEST LIST IS NOT THE BLAST RADIUS.** `h2__record.wat` was GREEN before a codemod
touched it. It was recovered only by sweeping all 1119 fixtures rather than the 795 that were red.
When you finish, re-check more than you changed.

**③ DO NOT GENERALIZE FROM ONE SAMPLE.** The previous rider sampled 1 of 17 failures, generalized
the bucket as "not mine", then caught itself and retracted — in the same report where it had just
proved rule ①. Each of the 35 probes below gets its own diagnosis.

## The worklist

## The 35 failing probes, with failure counts and whether they embed wat source
```
  16  other       tests/services/probe_arc278_recv_outcome_wall.rs
  10  other       tests/services/probe_arc278_call_context.rs
   8  other       tests/types/enums.rs
   8  other       tests/services/probe_arc278_sift_rules_arena.rs
   8  other       tests/services/probe_arc278_sift_rules.rs
   8  other       tests/services/probe_arc278_sift_logs.rs
   8  EMBEDS-WAT  tests/services/probe_arc278_peers_bijection.rs
   6  other       tests/services/probe_arc278_service_max_frame_bytes.rs
   6  other       tests/resolve/probe_arc255_register_variant_is_its_own_door.rs
   6  EMBEDS-WAT  tests/diagnostics/probe_arc241_stone10_remedy.rs
   4  other       tests/services/probe_arc278_arming_is_internal_only.rs
   4  other       tests/kernel/test.rs
   4  other       tests/cli/wat_mcp.rs
   4  EMBEDS-WAT  tests/wat_lang/probe_arc241_stone15_zombie_purge.rs
   4  EMBEDS-WAT  tests/function/wat_arc170_closure_extraction.rs
   2  other       tests/types/parametric_enum.rs
   2  other       tests/services/probe_arc278_journal_query_logs.rs
   2  other       tests/services/probe_arc278_journal_logs_on_process.rs
   2  other       tests/services/probe_arc278_dead_child_speaks.rs
   2  other       tests/services/probe_arc272_rs2_crash_surfaces_to_client.rs
   2  other       tests/services/probe_arc170_gapj_each_kwargs.rs
   2  other       tests/services/probe_arc170_c2_strike1_mixed.rs
   2  other       tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head.rs
   2  other       tests/program/probe_arc259_peer_env_install.rs
   2  other       tests/lint/wat_scripts_fixes_load.rs
   2  other       tests/lint/every_ungated_wat_checks.rs
   2  other       tests/diagnostics/probe_diagnostic_c3_macro_emits_record_def.rs
   2  EMBEDS-WAT  tests/value/wat_arc221b_keyword_dispatcher_completeness.rs
   2  EMBEDS-WAT  tests/types/typed_if_match.rs
   2  EMBEDS-WAT  tests/services/probe_arc170_c2_mixed_macro.rs
   2  EMBEDS-WAT  tests/reflection/probe_stone_metadata_of_whole_row.rs
   2  EMBEDS-WAT  tests/program/probe_arc259_env_peer_kind.rs
   2  EMBEDS-WAT  tests/process/signal_kill_produces_close_outcome_signaled.rs
   2  EMBEDS-WAT  tests/process/probe_supervisor_select_lost.rs
   2  EMBEDS-WAT  tests/lint/one_variant_separator.rs
```

## Four known sub-classes — and one that is not a defect

**⑧ Wat source embedded in RUST string literals** (the `EMBEDS-WAT` rows). No `.wat`-path codemod
can reach these; they are scaffolding inside `.rs` files. Same rename, same discriminator: a `::` is
a variant separator only when the left side is an enum and the right is one of its variants.

**`probe_arc241_stone10_remedy.rs`** — the remedy ENGINE now correctly proposes `":my::Status.Ok"`
(one candidate, score 3); the test hardcodes the old two-candidate set (`"::Ok"` score 1, `"::Error"`
score 5). The engine outgrew the test. Re-capture the expectation from the engine; do not bend the
engine to the test.

**`probe_arc255_register_variant_is_its_own_door.rs`** — ⚠ **NOT A REPAIR.** Its two collision
fixtures assert that a `defn` and an enum variant wanting the same name raise `DuplicateDefine` in
either order. Stone ③b-i made H-1 absolute for a DECLARED name, so a user can no longer TYPE
`:my::app::Foo.Bar` at all: the collision is now **structurally unconstructible**, which is the rung
ABOVE catching it. Rewrite those rows to assert the new truth — the attempt is refused as
`DottedName` before collision-detection is reached — and say so in the test's own doc comment. The
sibling `__dotted_defn`/`__dotted_variant` fixtures already test that refusal and must stay green.

**`probe_diagnostic_c3_macro_emits_record_def.wat`** — a `defmacro` builds a variant name at expand
time via `(:wat::string::concat base-str "::Op::Go")`. A hardcoded separator inside a STRING, which
`rename-keyword-exact` cannot touch by design. This is the same shape as `wat/service.wat`'s
seventeen, and the substrate now has the door for it: **`:wat::runtime::compose-variant`**.

**The remaining ~23 `other` rows are UNDIAGNOSED.** They are not assumed to be variant-spelling and
not assumed to be pre-existing. Diagnose each.

⚠ **"Pre-existing" has a zero point, and it is `7ccce48ba` — which floored 5334/5334 GREEN.** Two
riders in a row called their residue pre-existing after checking a POST-flip baseline. If you claim
a failure predates the flip, name the commit you checked it against.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** A fix requires changing substrate behaviour rather than a test's expectation. Report it.
A test that has outgrown its engine is re-captured; an engine that is wrong is a finding.

**STOP-2.** A `.wat.bad` fixture stops failing, or starts failing for a different reason. Those
exist to be REFUSED; a change in WHY is a finding about the checker.

**STOP-3.** You find a NINTH class — a failure shape that is neither variant spelling, nor an
outgrown expectation, nor one of the four above. Report it with a verbatim instance. Seven of the
eight classes in this landing were found exactly this way.

**STOP-4.** A rename would move a `::` whose left side is not an enum. Report it.

## What to run

`cargo build --release`, `./target/release/wat --check` on any fixture you touch, and scoped
`cargo nextest run --release -E 'binary_id(wat::<dir>)'` per directory. **Do not run
`scripts/floor.sh` and do not run clippy** — the orchestrator measures centrally, once. Run every
command in the FOREGROUND and block on it: no `run_in_background`, no Monitor, no "I'll wait" —
ending your turn ENDS you, and this has cost five riders in this campaign. Do not commit.

## Report

Per probe: what was wrong and what you did. Every STOP finding. Anything you were less than certain
about — and if you sample rather than verify, say which rows you sampled and which you did not.
