# STONE — ONE PARAM-SPEC: the `.wat` corpus codemod

> **Builder's ruling, 2026-08-29:** *"there is exactly one way to confer a parametric type. it is
> `:- [...]`. all others must die."*
>
> Stone 1 of three. Read `NOTE-a-parametric-literal-has-three-spellings-and-no-authority-names-all-three.md`
> first — it carries the ruling, the population, and why two prior migrations missed this.

## The work

A recorded wat-fix codemod at `wat-scripts/fixes/one-param-spec.wat` that rewrites every heretical
param-spec in the `.wat` corpus to `:- [...]`.

```
(:wat::core::Vector :wat::core::i64 1 2 3)      ->  (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
(:wat::core::HashMap :k :v k1 v1)               ->  (:wat::core::HashMap :- [:k :v] k1 v1)
(:wat::core::Vector [:wat::core::i64] 1 2 3)    ->  (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
```

**Population, measured at `27923cb2c`: 1474 bare + 23 unmarked = 1497 sites in `.wat`.** The `.rs`
doc-comment sites (218) are stone 2; the checker wall is stone 3. **Do not touch `src/`.**

⚠ **`:wat::core::fn`'s `[...]` is its PARAMETER LIST, not a param-spec.** 1053 sites. Rewriting one
of them is a catastrophic false positive. It is the single most likely way this stone goes wrong.

## ★ THE CONTRACT DECISION — arity comes from a SOURCE, never from counting leading keywords

A naive "the leading keywords are the types" rule is **wrong**:
`(:wat::core::Vector :wat::core::keyword :a :b :c)` is ONE type param and THREE keyword *values*.

**Two positions, and only one of them is hard:**

- **TYPE position** — `-> (:wat::core::Option :wat::core::String)` — there are no trailing values, so
  **every argument is a type param.** Unambiguous; rewrite directly.
- **LITERAL position** — `(:wat::core::Vector :wat::core::i64 1 2 3)` — the first N args are types
  and the rest are values, and **N must come from a source**:
  1. **User types declare it themselves**: `(:wat::core::defrecord :wat::cache::Entry :- [K V] …)`
     → arity 2. Collect these in a first pass over the corpus.
  2. **Substrate parametrics** have no wat declaration. Carry a **NAMED table** in the codemod —
     `Vector 1 · HashSet 1 · PersistentVector 1 · Option 1 · HashMap 2 · PersistentMap 2 · Result 2`
     — each entry justified in a comment, and **`Tuple` deliberately absent** (its param count
     equals its value count; the bare form cannot be disambiguated — 3 sites, REPORT them).

⛔ **A head in neither source is REPORTED, never guessed.**

## ★ THE ORACLE — the checker, not the codemod's own confidence

**A wrong split fails `--check`.** `(:wat::core::Vector :- [:wat::core::keyword :a] :b)` does not
type-check, because `:a` is not a type. So the migration validates itself against the instrument
that already knows every arity. **Run `target/release/wat --check` over every rewritten file** — that
is the acceptance, not the diff looking plausible.

## ⛔ SILENCE IS THE FAILURE MODE THIS STONE MUST NOT HAVE

The three spellings exist because a migration answered its own question completely and never said
what it had not looked at. **This codemod REPORTS every site it did not rewrite** — unknown head,
ambiguous arity, a shape it did not recognise — to stderr, with file and line, and the count goes in
the report. A partial migration that looks total is exactly how we got here.
`[[feedback_a_census_of_a_name_must_ask_every_rendering]]`

## How to run it — R21, and the sibling to copy

`wat/fix.wat` is the framework (`fix-source` walks via `read-string` → `with-children`; span-faithful
edits via `ast-span`/`ast-end-span`/`fix-text-apply`). **Copy the shape from
`wat-scripts/fixes/angle-brackets-to-binder.wat`** — the closest sibling, same target form, and its
header explains why it carries its own renderer.

```bash
# DRY RUN FIRST — on a /tmp copy, then diff. Never straight at the corpus.
printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/one-param-spec.wat
```

Idempotent: a second run must produce zero changes. **This stone needs NO bootstrap/stash dance** —
it adds no `:wat::fix::` verb and ships no `src/` change; the wall is stone 3.

★ **The fix corpus is inside the population it fixes.** `positional-to-kwargs.wat` itself contains
the bare form. `wat-scripts/**` is in scope, and the codemod will rewrite its own siblings.

## STOP triggers — each REJECTS.

1. **A `:wat::core::fn` parameter list is rewritten.** Catastrophic. Stop, report, revert that file.
2. **A head's arity cannot be sourced.** Report it; do not guess, do not infer from the site.
3. **`--check` fails on any rewritten file** and the cause is the rewrite. Stop and report the form.
4. **You reach for `sed`/python or a hand-edit.** R21: the codemod is the tool. If it cannot express
   the rewrite, that is a finding about `fix.wat`, not a licence to edit by hand.
5. **You are about to touch `src/`.** Stones 2 and 3, not this one.

## Acceptance

```
 0. ★ YOUR OWN POPULATION COUNT before writing anything — bare and unmarked, per head, with the
      `:wat::core::fn` exclusion stated and its count shown. Disagreement with my 1474/23 is a finding.
 1. ★ THE DRY RUN, ON A /tmp COPY, DIFFED. Paste a representative hunk for each of: a literal-position
      rewrite, a type-position rewrite, an unmarked-bracket rewrite, and a NESTED one.
 2. ★ ZERO `:wat::core::fn` FORMS TOUCHED — prove it: `git diff` filtered to lines containing
      `:wat::core::fn` must be empty across the whole corpus.
 3. ★ EVERY REWRITTEN FILE `--check`s CLEAN. Report the command and the file count.
 4. ★ THE REPORT OF WHAT WAS NOT REWRITTEN — every skipped site with file, line, and reason.
      `Tuple`'s 3 sites must appear here. A zero-skip run is a FINDING, not a success.
 5. ★ IDEMPOTENT — a second run changes nothing. Show it.
 6. ★ POPULATION AFTER: bare 0, unmarked 0, by your own row-0 command.
 7. ★ THE CODEMOD IS COMMITTED at `wat-scripts/fixes/one-param-spec.wat` as the recorded migration,
      with a header naming its two arity sources and what it refuses.
 8. ★ `git diff --stat src/` EMPTY. Say it.
 9. cargo build --release --all-targets — clean.
10. cargo nextest run --release -E 'test(wat_scripts) + test(lint) + test(check)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.

## Report back with

Your row-0 population count. The four diff hunks. The `:wat::core::fn` proof. The `--check` result.
**The full list of sites not rewritten, with reasons.** The idempotency run. The after-counts. Then
the honest deltas — especially any shape the codemod could not express.
