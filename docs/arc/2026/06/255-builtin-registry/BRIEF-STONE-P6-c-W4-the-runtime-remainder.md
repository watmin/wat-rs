# STONE P6-c-W4 — the three that GROW a frozen ledger, and what that ledger really means

> Wave 4 closes `:wat::runtime::`. Three verbs — and the first wave to make a frozen ledger GROW.
> Read `BRIEF-STONE-P6-c-W3-runtime-reflection.md` first.

## The three

```
:wat::runtime::metadata-of      eval_metadata_of       arity 1    167 corpus call sites
:wat::runtime::field-names-of   eval_field_names_of    arity 1      8 corpus call sites
:wat::runtime::field-types-of   eval_field_types_of    arity 1      5 corpus call sites
```

W3 cut these because homing them makes `FROZEN_CHECKER_DEBT_LEDGER` grow. That is this stone.

## ★ THE FINDING THAT RESHAPES THE LEDGER'S MEANING — verify it first

The ledger's criterion is `check_env.get(fqdn).is_none()`. I and this arc's prose have repeatedly
called that *"verbs the checker does not know"*. **It is not.** It means **"no `TypeScheme`
registered"** — and two of these three ARE typed, by hand-written special-case inference inside
`infer_list` (`src/check.rs:2543`; `field-names-of` at `:3570`, `field-types-of` at `:3596`, each
with its own declared shape in a comment).

So the ledger will grow by three, and **two of those three entries are not untyped.** Their ledger
lines must SAY SO — `metadata-of` genuinely has no check-side treatment at all (0 mentions in
`check.rs`), the other two have inference and merely lack a scheme. A ledger that records all three
identically overstates the debt and rots into a list nobody trusts, which is the exact failure its
own header warns about. `[[feedback_a_gate_freezes_names_never_a_count]]`

⚠ **This matters far past these three.** ~37 of the remaining population sit on the same criterion.
If the campaign reads "absent from `check_env`" as "unchecked", it will mis-size the debt for every
one of them.

## ⛔ Nothing about checking may change

`metadata-of` has **167 corpus call sites**. Homing it must leave checking exactly as it is: it gains
a registry entry and a ledger line, **not a `TypeScheme`**. Writing one would type-check 167 sites
for the first time — that is arc 255's thesis (killing the blanket-accept at `walk.rs:268`) arriving
one verb at a time, and it is **not this stone's work**. Same for the other two.

**If you find yourself editing `src/check.rs`, STOP.**

## The work

1. Home all three: `#[wat_intrinsic]` + full `///` block + real arity, inline arity guard deleted.
2. Add all three to `FROZEN_CHECKER_DEBT_LEDGER`, **each line stating its true status** — no scheme
   AND no inference (`metadata-of`), versus no scheme BUT typed by `infer_list` (the other two,
   citing their line).
3. **Row 2 of W3 applies again, and these are still the reflection surface.** W3 found SEVEN doc
   lies in ten verbs, five of them a stale `:wat::holon::HolonAST` return type. Read each of these
   three against its body before carrying prose into a `///` block. `metadata-of`'s header currently
   says `-> (:Option :- [(HashMap :- [Keyword HolonAST])])` — **check that**.

★ **And a live follow-up you may confirm but must not fix:** the same stale `HolonAST` prose survives
at `check.rs:3436`, `check.rs:19790`, and 3× in `tests/reflection/wat_arc201_extract_arg_types.rs`,
which describes a pipeline through `holon_type_ast_to_wat_type_form` — **a function with zero
definitions anywhere.** W3 verified that. Out of scope; say if you see more.

## STOP triggers — each REJECTS.

1. **You are about to edit `src/check.rs`.** Nothing about checking changes here.
2. **A doc contradicts its body and you cannot tell which is right.** Report both; home nothing there.
3. **The debt ledger's gate goes red in a way adding three lines does not fix.** Report it.
4. **A purity you would have to guess.**

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: shape · return type · arity · dispatch sites · and for each, whether
      check.rs treats it (scheme? inference? neither?) with the line. Disagreements reported first.
 1. ★ THE LEDGER GROWS BY EXACTLY 3 — 50 → 53 — and each new line states its true check status.
      `checker_skip_debt_is_named_and_frozen` PASSES.
 2. ★ EVERY DOC READ AGAINST ITS BODY — matched or corrected, with before/after text.
 3. ★ ARITY PUBLISHED: `metadata-of` for all three (use the verb itself before homing; after
      homing it reports on itself — say so). Each matches the guard you deleted.
 4. ★ THE ARITY ERROR SURVIVES for each, same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after — and for `metadata-of` include a call from
      the CORPUS shape, not only a synthetic one, given its 167 sites.
 6. ★ CHECKING IS UNCHANGED. `git diff --stat src/check.rs` is EMPTY. Say it.
 7. ★ `KNOWN_UNREVIEWED` unchanged at 240 (the gate's own line, not a grep).
 8. ★ Population 128 → 125. Registry delta +3, with the anchored command from W3's brief.
 9. cargo build --release --all-targets — clean; warnings VERBATIM.
10. cargo nextest run --release -E 'test(runtime) + test(intrinsic) + test(reflection) + test(arc170)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table including each verb's check-side treatment and line. The three ledger lines
verbatim. Row 2's verdicts with before/after text. Arity and error quotes. `git diff --stat
src/check.rs`. Then the honest deltas — especially whether the ledger's criterion misled you
anywhere else.
