# STONE P2 — the special-form entry stops lying

> Row P2 of `WORKLIST-open-stones.md`. Findings **2** and **6** of
> `NOTE-an-absence-recorded-as-an-answer-the-class-behind-the-apply-defect.md`, which carries the
> disk citations. Two findings, **one reason to change**: the registry entry the special-form fold
> builds says two things that are not true, and both are read by the reflection surface.

## The two lies

**A. `show-source` returns `""` for a special form, while that verb's own shipped prose promises
otherwise.** Proven live: `(:wat::core::show-source :wat::core::if)` → `""`.

- The claim, `src/intrinsic/reflect.rs:218` — and this text IS `:wat::core::show-source`'s own
  `entry.prose`, so a wat caller meets it through `render-doc`/`metadata-of`:
  > *"Returns the FQDN keyword form string for primitives and special forms (no source available)."*
- The code: `reflect.rs:241` consults `lookup_entry` FIRST and returns `entry.source`
  unconditionally on a hit; `src/intrinsic/mod.rs` hardcodes `source: ""` for every
  `Kind::SpecialForm` in the fold.
- The honest text already exists and is **unreachable** for a registered special form —
  `reflect.rs:~269`, the `Binding::Primitive | Binding::SpecialForm | Binding::Type` arm:
  `";; {name} — substrate primitive (no source available in this context)"`. The registry hit at
  :241 returns before anything can reach it.

**B. `metadata-of` reports `:arity -1` for a form that declares three fixed arguments.**
`src/intrinsic/mod.rs`'s fold hardcodes `arity: Arity::Variadic, // special forms handle their own
arity`, ignoring `submission.args` — which is set from the same struct literal, two lines below.
`:wat::core::if` declares exactly three `@arg`s (`src/intrinsic/special/control_flow.rs:14-16`) and
`metadata-of` says `-1`. `render-doc` gets it RIGHT from the same data (`reflect.rs:349-356` derives
its "Syntax:" line from `entry.args`), so the registry already holds what is needed.

## The ONE CONTRACT DECISION, and the trap inside it

⛔ **`arity` MUST NOT be derived as `Exact(args.len())`. That would ship a WORSE number than the one
it replaces.** Measured:

```
:wat::core::if    declares 3 @args, no @syntax   ->  Exact(3)     ✓
:wat::core::let   declares @syntax, ZERO @args   ->  Exact(0)     ⛔ WRONG — `let` is genuinely variadic
```

`src/intrinsic/special/binding.rs` gives `let` a `@syntax (let [<binder> <expr> ...] <body>+)` line
and no `@arg` directives at all. **The rule is:**

```rust
// Exact(N) only when the form actually enumerated its arguments and none of them is a rest param.
// Anything else — a rest param, or a form that declares its shape as @syntax instead of @arg —
// is Variadic, which is what `let` is and what the old hardcode accidentally got right for it.
let arity = match submission.args {
    args if !args.is_empty() && !args.iter().any(|(_, _, _, is_rest)| *is_rest) =>
        Arity::Exact(args.len()),
    _ => Arity::Variadic,
};
```

This mirrors what `#[wat_intrinsic]` already does for ordinary handlers
(`crates/wat-macros/src/wat_intrinsic.rs:653-657`: variadic ⇒ `Arity::Variadic`, else `Arity::Exact(n)`).

⛔ **And `show-source` must dispatch on `entry.kind`, NOT on `entry.source.is_empty()`.** An empty
capture from a genuine intrinsic would then be indistinguishable from a special form's structural
absence — which is precisely the class this whole NOTE is about. **Do not fix an
absence-read-as-an-answer with another absence-read-as-an-answer.**

## Rooms — verified against `4fad41b35`

```
src/intrinsic/mod.rs   the special-form fold: `arity: Arity::Variadic` and `source: ""`,
                       in the `for submission in inventory::iter::<SpecialFormSubmission>` loop.
                       `args: submission.args` is set in the SAME struct literal — the data is there.
src/intrinsic/reflect.rs:218      the CLAIM — show-source's own @doc prose
src/intrinsic/reflect.rs:241      the unconditional `entry.source` return you gate
src/intrinsic/reflect.rs:~269     the honest fallback text, currently unreachable for special forms.
                                  ⚠ It also serves Binding::Primitive and Binding::Type — do NOT
                                  delete it; those paths are live.
src/intrinsic/reflect.rs:349-356  render-doc deriving from entry.args — the worked precedent for B
src/intrinsic/special/control_flow.rs:14-16   `if`: three @args, no @syntax
src/intrinsic/special/binding.rs              `let`: @syntax, zero @args   ← the trap
src/intrinsic/mod.rs:221          the arg tuple: (name, ty, desc, is_rest)
```

## Blast radius

`src/intrinsic/mod.rs` (the special-form fold) and `src/intrinsic/reflect.rs` (one gate in
`show-source`). Nothing else. No macro change, no runtime change, no new field, no change to the
intrinsic fold.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A special form declares NEITHER `@arg` nor `@syntax`.** Then `Variadic` is a guess and the form
   is under-documented. STOP and name it. (Measured today: only `if` and `let` exist, and each
   declares one of the two — so this is cover, not an expected case.)
2. **`show-source`'s new text differs from the existing fallback's.** Two paths saying two things
   about one situation is the defect wearing new clothes. Reuse the fallback's exact wording, or
   route to it.
3. **You gate on `source.is_empty()` instead of `kind`.** See the contract decision.
4. **Any INTRINSIC's `metadata-of` output changes.** This stone touches the special-form fold only.
   The 380 intrinsics must report exactly what they report today.
5. **`render-doc`'s output changes for anything.** It already derives correctly; if your change
   moves it, you have changed the wrong thing.

## Acceptance — run each, report the actual output

```
 0. ★ THE TWO LIES, BEFORE AND AFTER. One scratch .wat under wat-scripts/scratch-pad/ (loader-gated,
    must `--check` clean) printing, for BOTH `:wat::core::if` and `:wat::core::let`:
      (:wat::core::show-source <form>)
      (:wat::runtime::metadata-of <form>)   — show the :arity field
    Paste the run BEFORE and AFTER. Expected after:
      if    show-source -> the honest "no source available" text, naming `:wat::core::if`
            :arity     -> 3
      let   show-source -> the same honest text, naming `:wat::core::let`
            :arity     -> -1  (STILL variadic — and that is CORRECT; `let` declares @syntax,
                               not @args. If `let` reports 0, the trap in the contract was hit.)

 1. ★ AN INTRINSIC IS UNTOUCHED. `show-source` and `metadata-of` on a registered intrinsic
    (`:wat::i64::+`, and one migrated by O-iv-b such as `:wat::map::length`) — byte-identical
    before and after. This is STOP-4's positive form.

 2. ★ PROVE EACH BY BREAKING ITS DOOR, SEPARATELY. Revert the `kind` gate alone → show-source
    returns `""` again. Revert the arity derivation alone → `if` reports -1 again. Restore both.
    Two findings, two independent proofs — one combined revert proves neither.
    Confirm each edit LANDED before reading its output.

 3. ★ render-doc IS UNCHANGED. `(:wat::core::render-doc :wat::core::if)` byte-identical before and
    after — it already derived correctly and must keep doing so.

 4. cargo build --release --all-targets — clean.

 5. cargo nextest run --release -E 'binary_id(wat::reflection) + test(show_source) + test(metadata)'
    Report the Summary lines verbatim. ⚠ If a test in that set was GREEN before and is RED after,
    it may be a golden pinning the OLD wrong value — that is a finding, not a licence to edit the
    golden. STOP and report which assertion, with its text.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers before your
  turn ends — a rider on this chain was lost mid-strike and left an implementation with no evidence,
  and every one of its rows had to be re-run from scratch.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- **No `git stash`, in any form** — not to capture a baseline, not to compare. Use
  `git show HEAD:<path>` for a pre-image. The last rider reached for stash to re-measure something
  it had already captured; it did no damage and reported it honestly, and it is still forbidden.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas — what surprised you,
what this brief got wrong, what you had to decide that it did not settle. Every rider on this chain
has caught a real defect in an orchestrator brief; that is the most useful thing you can hand back.
