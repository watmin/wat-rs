# BRIEF — STONE expand-T2: `@ExpandTime` parses and reaches the entry

Read `DESIGN-STONE-expand-t2-the-axis-is-declarable-and-carried.md` first.

## The work, one paragraph

The `ExpandTime` enum exists (`0625c6b2c`). **Make it declarable and make it arrive.** A doc comment
gains `@ExpandTime <Variant>`; both doc structs carry it; both proc-macros turn it into a token;
both submission structs and `IntrinsicEntry` hold it. **`@Totality` is your exact template** —
every site below is one where `totality` already appears.

## Read in order — every site named, none left to discover

```
crates/wat-doc/src/lib.rs:~153     DocComment's purity/determinism/totality/category — join them
crates/wat-doc/src/lib.rs:~250     DocSpecialForm — a SIBLING type with its own parse fn.
                                   BOTH need the field; both resolution points need the default.
crates/wat-doc/src/lib.rs:~678     `parse`'s resolution point       — DEFAULT, do not error
crates/wat-doc/src/lib.rs:~996     `parse_special_form`'s           — DEFAULT, do not error
crates/wat-doc/src/lib.rs:~221     DocError variants — add InvalidExpandTimeVariant
crates/wat-macros/src/wat_intrinsic.rs      `totality_token` — copy its shape
crates/wat-macros/src/wat_intrinsic.rs      ★ render_doc_error — A THIRD EXHAUSTIVE MATCH.
                                   Your new DocError variant WILL break it with E0004. Expected.
crates/wat-macros/src/wat_special_form.rs   the twin token fn. ⚠ Its error path is
                                   format!("{:?}", e), NOT a match — no third site here.
src/intrinsic/mod.rs:~254/~287/~401  IntrinsicSubmission · SpecialFormSubmission · IntrinsicEntry
src/intrinsic/mod.rs               BOTH submission -> entry conversions
crates/wat-macros/*                both `inventory::submit!` emit literals
```

## Implementation sketch

```rust
// wat-doc, alongside totality_val
let mut expand_time_val: Option<ExpandTime> = None;
// … in the directive loop, mirroring @Totality:
//   duplicate -> DocError::DuplicateSingleton
//   unknown   -> DocError::InvalidExpandTimeVariant { got }
// … at BOTH resolution points, DEFAULT rather than error:
let expand_time = expand_time_val.unwrap_or(ExpandTime::Unreviewed);

// both proc-macros, exhaustive, NO wildcard arm:
wat_doc::ExpandTime::Legal       => quote! { ::wat_doc::ExpandTime::Legal },
wat_doc::ExpandTime::RuntimeOnly => quote! { ::wat_doc::ExpandTime::RuntimeOnly },
wat_doc::ExpandTime::Preserving  => quote! { ::wat_doc::ExpandTime::Preserving },
wat_doc::ExpandTime::Unreviewed  => quote! { ::wat_doc::ExpandTime::Unreviewed },
```

★ **Placement in a doc block: after `@Totality`, before `@Category`** — so the four property axes read
as a block. `:wat::i64::/` shows the column alignment.

## Blast radius

`crates/wat-doc/`, `crates/wat-macros/`, `src/intrinsic/mod.rs`, and the ONE verb you annotate for
row 4. Nothing else.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **You are about to make `@ExpandTime` mandatory** (a `Missing*` error, or removing the default).
   T3, and a ~433-site compile break. STOP.
2. **You are about to annotate a second real verb.** One, for the carriage proof. T4a transcribes
   the rest from the existing allow-list. STOP.
3. **You are about to touch `is_expand_time_legal`.** It keeps its hand-list until T4. STOP.
4. **A wildcard `_ =>` would make a proc-macro match compile.** The exhaustiveness IS the
   mirror-drift protection (`E0004`). STOP and report what did not fit.
5. **The floor moves beyond your new tests.** Nothing reads the field yet, so nothing should
   change. STOP.

## Acceptance

```
 0. ★ PRE-CHECK: name both resolution points and confirm DocSpecialForm is a sibling type, not
      the same struct. Report BEFORE editing.
 1. ★ ALL FOUR VARIANTS PARSE, one test each; ABSENT yields `Unreviewed`, tested explicitly,
      on BOTH doc structs.
 2. ★ Duplicate is `DuplicateSingleton`; unknown is `InvalidExpandTimeVariant` whose rendered
      message NAMES ALL FOUR legal values. Quote it verbatim.
 3. ★ CARRIAGE, TWO-SIDED, on REAL verbs:
        :wat::core::fresh-symbol  declares `@ExpandTime Legal`  -> entry reads Legal
        a verb declaring nothing                                -> entry reads Unreviewed
      One-sided proves nothing: only-Legal passes if the field is hard-wired; only-Unreviewed
      passes if carriage is broken and everything defaults.
 4. ★ BREAK THE DOOR: replace the spliced token with a hard-coded `Unreviewed` in the intrinsic
      macro, show row 3's first assertion fail, restore. Quote the failure.
 5. ★ Both proc-macro matches are wildcard-free. State that you checked both.
 6. ★ `git diff` touches only the named files. Say so.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo test --release -p wat-doc -p wat-macros, and
    cargo nextest run --release -E 'test(intrinsic) + test(macro)'
```

★ **Row 3 is the stone.** Rows 1–2 test a parser; only row 3 tests that the value survives into the
thing a consumer reads. A directive that parses perfectly and is dropped on the floor passes 1–2.

★ **Why `fresh-symbol`:** it is this axis's own witness — nondeterministic yet expand-time-legal —
so `@ExpandTime Legal` on it is a true, load-bearing claim rather than decoration.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your pre-check. The four parse results and both default-when-absent results. The error message
verbatim. **Row 3's two readbacks.** Row 4's failure and restore. Confirmation both matches are
wildcard-free and the diff is confined. Then the honest deltas — especially any site this brief
named wrongly, since it was written from another axis's scars.
