# BRIEF — STONE 1a-β-i-b: the dead `defstruct` arms die; the live ones stay and say why

`:wat::core::defstruct` is a stdlib `defmacro` that `expand_all` rewrites to `structtype`. Every
consumer that runs AFTER expansion therefore has a dead `defstruct` arm — and the two that guard
`eval-ast!` run on **unexpanded** AST and are load-bearing. Classify all of them by that one
question, remove the dead, and leave the live arms carrying the sentence that says why.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-beta-i-b-defstruct-is-pre-expansion-in-one-place-and-post-in-six.md`

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5124.

## ★★ The gate the DESIGN asked for ALREADY EXISTS — verify it, do not build it

The design specified a new probe pinning the pre-expansion refusal. **Two tests already do it**, and
both migrated ONTO `defstruct` deliberately, saying so in their own comments:

```
src/freeze.rs:2253            eval_refuses_define
        asserts EvalForbidsMutationForm { head } == ":wat::core::defstruct"
        — pins freeze::is_mutation_form's arm
tests/value/wat_eval_result.rs:139
        asserts the message "eval refused mutation form: :wat::core::defstruct"
        — pins runtime::is_mutation_head's arm
```

Your job on that item is to **cite them, confirm each still passes untouched, and state which arm
each one pins.** Add a probe only if you find an arm with no coverage — and say plainly which.

## Read in order

1. **The DESIGN**, especially the two-pole probe evidence and the ⬜ MEASURE rows.
2. **`wat/core.wat:2030`** — `(:wat::core::defmacro :wat::core::defstruct …)`. The macro is the
   whole reason this stone exists; read what it rewrites to.
3. **`src/freeze.rs:1900`** (`is_mutation_form`'s arm — **KEEP**) and **`:1959`**
   (`is_liftable_declaration_head`'s arm — **REMOVE**). They are fifty lines apart and spell the
   arm identically. ⚠ A prior sabotage of mine edited the wrong one of these two and read the
   resulting green as a finding.
4. **`src/types.rs:4131`** — `parse_type_decl`'s `"defstruct" =>` arm, and **`:3999`**
   `classify_type_decl`'s.
5. **`src/types/defstruct.rs:520`** (`parse_defstruct`) and **`:29`** (`validate_defstruct_arity`).
6. **`src/intrinsic/mod.rs`**'s `liftable_declaration_head_missing_and_foreign` — its
   `domain.len() == 9` guard and its MISSING assertion both move.
7. **`tests/macros/probe_declaration_form_lift.rs:74`** — `probe_liftable_declaration_head_covers_all_nine_keywords`.
   It asserts `defstruct` is covered; it moves to eight, and its NAME moves with it.

## The work

### 1 — classify every `:wat::core::defstruct` site by ONE question

> **Can this code run before `expand_all`?**

`grep -rn '":wat::core::defstruct"' src/` and answer it per site. **POST → the arm is dead, remove
it. PRE → keep it, and write the reason at the site**, in one sentence, so the next sweep cannot
mistake it for a leftover of this one.

The design's table is a **prediction**. Two rows are marked ⬜ MEASURE —
`declare/parse.rs`'s `is_struct_form` (callers in `preregister.rs`) and `closure_extract.rs`'s
`walk_free_symbols` arm. **Establish which side of `expand_all` each runs on before touching it**,
and report how you established it.

### 2 — remove the dead functions

`parse_defstruct` and `validate_defstruct_arity` (its only caller), plus the arms that reach them.

⚠ **Most of `src/types/defstruct.rs` is LIVE and the FILE is not the unit.**
`parse_defstruct_metadata` and `parse_aggregate_fields_with_splices` are called from
`src/types.rs:4678`/`4684` via `parse_aggregate`, which `parse_structtype` uses. Leave a retirement
comment in the shape the prior stones used.

### 3 — the meter and the probe follow the domain

`domain.len()` 9 → 8, MISSING 4 → 3 (`def`·`defalias`·`defmacro`), and
`probe_liftable_declaration_head_covers_all_nine_keywords` becomes eight — **name and body**. Record
in its doc WHY `defstruct` left: not "it was wrong to be there", but "it is a macro; post-expansion
no such head survives, so the arm could never fire."

## Blast radius

`src/freeze.rs` (one arm removed, one kept + annotated) · `src/types.rs` (two arms) ·
`src/types/defstruct.rs` (two fns) · `src/intrinsic/mod.rs` (the meter's two numbers) ·
`tests/macros/probe_declaration_form_lift.rs` · possibly `src/declare/parse.rs` and
`src/closure_extract.rs` **pending your measurement**. No `.wat` change. No registration.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `is_mutation_form` and `is_mutation_head` KEEP their `defstruct` arm.** They guard
`eval-ast!`, which evaluates AST that was never macro-expanded. Measured live:
`(:wat::eval-ast! '(:wat::core::defstruct …))` answers *"eval refused mutation form:
:wat::core::defstruct"*. **Removing those two arms deletes a real refusal**, and a half-swept name is
exactly how those two functions came to disagree with each other in the first place.

**⛔ STOP-2 — a site you have not placed relative to `expand_all` is not a site you may touch.** The
design's table is my prediction, not a measurement. If you cannot establish a site's side, STOP and
report it as unresolved.

**⛔ STOP-3 — the FILE `src/types/defstruct.rs` is not dead.** Remove two functions, not a module. If
removing them appears to orphan anything else, report it rather than widening.

**⛔ STOP-4 — do not touch `def`/`defalias`/`defmacro`, and do not flip or delete any hand-list.**
MISSING is 3 after this stone, not 0. The consumer flip is 1a-β-ii.

**⛔ STOP-5 — do not weaken the meter to make a number.** `domain.len()` and MISSING move because the
SOURCE changed. If either has to be edited for any other reason, that is a finding.

**STOP-6 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. also remove `defstruct` from `is_mutation_form` → which test fires, and with what message?
2. also remove it from `is_mutation_head` → which test fires?
3. leave `parse_type_decl`'s `"defstruct"` arm in place after removing `parse_defstruct` → what does
   the compiler say? (proves the arm and the fn are one unit)

## Report

The classification table — **every site, its side of `expand_all`, and HOW you established it**
(especially the two ⬜ MEASURE rows) · the removed arms and fns verbatim · **the kept arms' new
sentences verbatim** · the two existing pin-tests cited, with which arm each pins · the meter's two
numbers before/after · the renamed probe · the three sabotage predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-i-the-type-declaration-family.md` — same report shape. Its STOP-5 is why this
stone exists.
