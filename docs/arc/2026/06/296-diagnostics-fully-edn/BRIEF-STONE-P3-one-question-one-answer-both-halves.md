# BRIEF — STONE P-3: one question, one answer — both halves

## The work, in one paragraph

Two positions still disagree with `resolve` about whether a name is a type. **Half 1:** the
annotation wall counts a *stdlib* `use!` as the user program's own, so an annotation is accepted
where the identical name in call-head position is refused — narrow the wall's reference set to the
declaring scope. **Half 2:** `is-type?` cannot see `subtype_edges`, so it denies a derive marker
the wall accepts as a bound — add the disjunct. Neither half changes what is a type; both change
who is asked.

## Read in order

1. `tests/types/probe_arc296_p3_one_question_one_answer.rs` — **the committed probe. Read it
   first.** Three controls green, two subjects `#[ignore]`d; its header carries every measurement.
2. `docs/.../DESIGN-STONE-P3-one-question-one-answer-both-halves.md` — the rule, the scope
   derivation, and what is explicitly out of scope.
3. `src/freeze/env.rs:210-221` — **stdlib** `use!` collection into `use_decls`.
   `src/freeze/env.rs:252-274` — **user** `use!` collection into `user_use`, the seed into
   `TypeEnv`, then the wall call. Both sets already exist and are already distinct; this stone
   does not create them.
4. `src/check.rs` `validate_named_type_annotations` — the wall. It iterates `env.iter()` and
   `symbols.functions_iter()`, and **each entry carries its NAME**, which is the scope.
5. `src/resolve/walk.rs:104-124` — the call-head coverage rule this stone makes the wall match.
   Note it is reached from a pass over user residue; that is why it is already scope-correct.
6. `src/reflect/verbs.rs` `eval_is_type` — half 2, one disjunct.
   `src/types.rs:847` `is_subtype_parent` — already exists, already used by the wall.

## Implementation sketch

**Half 1.** Pass both sets to the wall and pick per declaration:

```
let scope_decls = if is_reserved_prefix(name) { &stdlib_use } else { &user_use };
… first_unknown_named_type(ty, &bound, env, scope_decls) …
```

⚠ `use_decls` is currently the MERGED set. Keep a stdlib-only set rather than reusing the merged
one for the reserved-prefix arm, so each scope is asked about exactly its own declarations.

⛔ **Write one line at that call site saying this is a SCOPE SELECTION, not the reserved-prefix
SKIP that RELAND-1 deleted.** Every declaration is still validated; only the reference set differs.
Without that line the next reader reads the blanket's return.

**Half 2.** In `eval_is_type`, add `|| types.is_subtype_parent(&type_kw)` alongside the existing
`contains` / `is_builtin_primitive` disjuncts. Nothing else.

## Acceptance

```
cargo nextest run --release -E 'test(p3_one_question)'   5 passed, 0 skipped   (both un-ignored)
cargo nextest run --release -E 'test(p1_annotation)'     10 passed, 0 skipped
cargo nextest run --release -E 'test(p2prereq)'          4 passed, 0 skipped
```

Per fixture, with the freshly built binary:

```
…__user_annotation_without_user_use.wat   --check EXIT 1, message names :rust::sqlite::Connection
…__user_annotation_with_user_use.wat      --check EXIT 0
…__stdlib_annotation_still_loads.wat      runs, stdout "loaded"
…__is_type_on_a_derive_marker.wat         stdout true
…__is_type_on_a_non_marker.wat            stdout false
```

## Blast radius

Measured at **zero real corpus files** — every non-stdlib `.wat` that annotates a `:rust::` type
carries its own `use!`. If a corpus file goes red, that is a finding worth reporting verbatim, not
a number to absorb: the census said it should not happen.

Touching: `src/freeze/env.rs`, `src/check.rs` (the wall's signature and its per-entry selection),
`src/reflect/verbs.rs` (one disjunct). No change to `use!`'s grammar, to `resolve`, to `derive`, or
to the hand-list.

## STOP triggers — each is a REJECTION

**STOP-1.** If `the_stdlib_still_loads` goes red — STOP immediately. Scope-awareness is drawn wrong
and nothing loads; report the verbatim failure. Do not adjust the fixture and do not widen the
stdlib arm back to the merged set to make it pass.

**STOP-2.** If half 1 needs a `continue`, an allow-list, or any arm that skips validating a
declaration rather than choosing its reference set — STOP. That is the blanket returning under a
new name, and it is the one shape this stone must not take.

**STOP-3.** If `a_name_that_was_never_derived_to_is_not_a_type` goes red — STOP. Half 2 over-reached
into accepting names that are not parents in any edge.

**STOP-4.** If any corpus `.wat` outside the probe fixtures refuses after half 1 — STOP and report
the file list verbatim. The census predicted zero; a non-zero answer means the census's population
was wrong and the orchestrator re-plans.

**STOP-5.** If half 2 appears to require `derive` to validate its marker — STOP and report. That is
a ruled-out-of-scope question and the verb must agree with the wall regardless of how it is later
ruled.

## Tier

You edit and report. Run the three targeted probe binaries and the five per-fixture checks.
**The orchestrator runs the floor and clippy centrally, once, after the tree is quiescent.**
Report what only you can: which sites you inspected, whether any corpus file moved, and what
surprised you.
