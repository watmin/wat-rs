# BRIEF — STONE total-T2: `@Totality` becomes a declarable directive

Read `DESIGN-STONE-total-t2-the-axis-declarable.md` first. This brief is the strike path.

## The work, one paragraph

The `Totality` enum already exists — generated from `wat/runtime-meta.wat` by `wat_enum_from!`,
committed at `525dbdb5b` with a passing probe. **Make it declarable.** A doc comment gains
`@Totality <Variant>`; `DocComment` carries it; both proc-macros forward it into the registry entry.
`Determinism` is the exact structural twin — every site you touch is one where `Determinism`
already appears, and copying its shape is the intended method.

## Read in order — the rooms, and why each

```
wat/runtime-meta.wat            the Totality defenum + its four variants' prose. READ FIRST:
                                the variants' meanings decide every arm you write.
crates/wat-doc/src/lib.rs:135   `pub struct DocComment` — `purity`/`determinism`/`category` are
                                the siblings `totality` joins.
crates/wat-doc/src/lib.rs:651   `let determinism = determinism_val.ok_or(...)` — the parse-time
                                resolution point. Yours DEFAULTS instead of erroring.
crates/wat-doc/src/lib.rs:957   the SECOND resolution point (special forms). Both need it.
crates/wat-doc/src/lib.rs:221   `DocError::MissingDeterminism` / `InvalidDeterminismVariant` —
                                the error shape to mirror.
crates/wat-macros/src/wat_intrinsic.rs:787   the value -> quote! match, one arm per variant.
crates/wat-macros/src/wat_special_form.rs:75 the same match, again.
```

## Implementation sketch

```rust
// wat-doc: alongside purity_val / determinism_val
let mut totality_val: Option<Totality> = None;
// ... in the directive loop, mirroring "@Determinism":
//   duplicate -> DocError::DuplicateSingleton
//   unknown   -> DocError::InvalidTotalityVariant { got }
// ... at BOTH resolution points (:651 and :957), DEFAULT rather than error:
let totality = totality_val.unwrap_or(Totality::Unreviewed);

// both proc-macros, exhaustive, NO wildcard arm:
wat_doc::Totality::Total      => quote! { ::wat_doc::Totality::Total },
wat_doc::Totality::Partial    => quote! { ::wat_doc::Totality::Partial },
wat_doc::Totality::Preserving => quote! { ::wat_doc::Totality::Preserving },
wat_doc::Totality::Unreviewed => quote! { ::wat_doc::Totality::Unreviewed },
```

## Blast radius

`crates/wat-doc/` and `crates/wat-macros/` ONLY. **`src/` is not touched by this stone** — no
consumer changes, no Layer-1 field, and `src/rete/purity.rs` / `src/macros/eval.rs` /
`src/rete/vocabulary.rs` keep their hand-lists exactly as they are. `wat/runtime-meta.wat` is
already correct; it needs no edit.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **You are about to make `@Totality` mandatory** (a `MissingTotality` error, or removing the
   default). That is T3 and it is a 429-site compile break. STOP.
2. **You are about to add `@Totality` to a real verb.** Zero verbs are annotated by this stone.
   Fixtures only. STOP.
3. **You are about to edit any file under `src/`.** STOP.
4. **A wildcard `_ =>` arm would make one of the two proc-macro matches compile.** The
   exhaustiveness IS the mirror-drift protection (`E0004`). STOP and report what did not fit.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: name both parse resolution points you found, and say whether the
      special-form doc struct is the same type as DocComment or a sibling. Report BEFORE editing.
 1. ★ `@Totality Total` / `Partial` / `Preserving` / `Unreviewed` all parse, verified by a test
      per variant.
 2. ★ ABSENT `@Totality` yields `Totality::Unreviewed` — the default, tested explicitly.
 3. ★ A SECOND `@Totality` is `DuplicateSingleton`; an unknown variant is `InvalidTotalityVariant`
      whose message NAMES ALL FOUR legal values. Both tested.
 4. ★ BREAK THE DOOR, and keep the artifact as a test: prove `@Totality` actually reaches the
      registry — a fixture verb declaring `@Totality Partial` whose registry entry reads back
      `Partial`, NOT `Unreviewed`. A test that only proves parsing proves nothing about carriage.
 5. ★ Both proc-macro matches are exhaustive with NO wildcard. State that you checked both.
 6. ★ `git diff --stat src/` is EMPTY. Say so explicitly.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo test --release -p wat-doc — green, count reported.
```

★ **Row 4 is the load-bearing one.** Rows 1–3 test the parser; only row 4 tests that the value
survives into the thing consumers will read. A directive that parses and is then dropped on the
floor would pass 1–3.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your pre-check. The four parse results. The default-when-absent result. The two error results with
their message text verbatim. **Row 4's before/after readback.** Confirmation both matches are
wildcard-free, and that `src/` is untouched. Then the honest deltas — anything the brief's sketch
got wrong about the real shape of the code.
