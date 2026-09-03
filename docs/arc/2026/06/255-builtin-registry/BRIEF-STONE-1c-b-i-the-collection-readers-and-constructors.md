# BRIEF — STONE 1c-b-i: five collection readers and constructors enter the registry

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-b-i-the-collection-readers-and-constructors.md`

## The work, in one paragraph

Five `:wat::core::*` verbs have a literal dispatch arm and a literal checker arm but **no
`CheckEnv` scheme and no registry row**. Each arm does a little pre-processing and then delegates.
**For each: extract a thin per-name wrapper fn, annotate that wrapper with
`#[wat_intrinsic("<fqdn>")]` plus an argued doc block carrying all five closed-domain axes, delete
the literal arm, and apply what the ledger ratchets name.** 1,302 corpus call sites.

```
:wat::core::first             362 sites   rt:2545  -> eval_positional_accessor(…, ":…first",  0)
:wat::core::second            230         rt:2548  -> eval_positional_accessor(…, ":…second", 1)
:wat::core::third               5         rt:2551  -> eval_positional_accessor(…, ":…third",  2)
:wat::core::PersistentVector  483         rt:2887  -> split_type_param_bracket + eval_persistentvector_ctor
:wat::core::PersistentMap     222         rt:2876  -> split_type_param_bracket + eval_persistentmap_ctor
```

## ⛔ WHY WRAPPERS, NOT IN-PLACE ANNOTATION — read this first

The three accessors **share one implementation**, `eval_positional_accessor`, parameterised by
FQDN and index. `#[wat_intrinsic]` emits its shim as
`format_ident!("__wat_intrinsic_shim_{}", fn_name)` — **keyed on the fn identifier, not the
FQDN**. Three annotations on that one fn emit three shims with the same name and will not
compile, with an error naming a mangled symbol and never the real cause.

`[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]` records this and forecast this
exact moment. So write five small wrappers — `eval_first`, `eval_second`, `eval_third`,
`eval_persistentvector`, `eval_persistentmap` (or names of that shape) — each with the canonical
signature `(args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable) ->
Result<Value, EvalBreak>`, each forwarding exactly what its arm forwards today.

⛔ **The shared implementations are NOT touched.** `eval_positional_accessor`,
`infer_positional_accessor`, `eval_persistentvector_ctor`, `eval_persistentmap_ctor` keep their
current bodies and signatures. The two constructor wrappers must carry the arm's
`split_type_param_bracket` pre-processing **verbatim** — dropping it would change what a
`(PersistentVector :- [T] …)` call evaluates.

## Read in order

1. **`src/collection/transform.rs`** — six rows landed there at Stone 1c-a-i in the shape you
   want: `#[wat_intrinsic]`, argued axes, no body change. Copy it.
2. **`src/intrinsic/special/stream_lazy.rs`** — the thin-delegate precedent, for the wrapper shape.
3. **The five arms**, at the `file:line`s above. Line numbers are for finding, not for trusting.
4. **`src/check.rs`'s `infer_positional_accessor`** (~`:9437`) — read its doc before grading the
   accessors' `@Totality`. It states the polymorphism explicitly and says why a rank-1 scheme
   cannot express it.

## The five axes — argue each, from the code

⚠ Unlike the last two stones, **the type gate cannot help you here.**
`doc_arg_ret_types_match_checker_scheme` skips rows with no `CheckEnv` scheme, and none of these
five has one. Nothing will catch a wrong `@arg`/`@ret`. Ground every type in the checker arm that
actually types the call (`infer_positional_accessor` for the three; the `PersistentVector` /
`PersistentMap` arms in `check.rs` for the two) and say which line you read.

- `@Totality` for the accessors: read what happens on a short container and on a non-indexable
  one. An arity guard is outside totality's domain (existing rows state that carve-out); a miss
  that returns `Option::None` is total; a miss that raises is not. **Read, do not assume.**
- `@Totality` for the constructors: read `eval_*_ctor`'s real error paths, and what
  `split_type_param_bracket` does when the bracket is absent versus malformed.
- `@ExpandTime`: the authority is `src/macros/eval.rs` — `is_expand_time_legal`'s registry-first
  consult and its residue list. Grep each FQDN. ⚠ A name in **neither** is being silently refused
  today with no ruling ever made; this arc has found two already (`ann-form`, `apply`). If you
  find a third, say so — and grade the pole that describes what the verb actually is, not the one
  that preserves today's accident.
- ⚠ `declared_purity_vs_effectful_by_prefix_census` requires an `@Purity Effectful` row's
  namespace to be in `effectful_by_prefix`; `:wat::core::` is not. A grounded `Effectful` reading
  is **STOP-3**, not something to grade around.

`@Totality Unreviewed` is not available: `KNOWN_UNREVIEWED` must not grow.

## Blast radius

`src/runtime.rs` (five new wrapper fns + five doc blocks + five arm retirements) ·
`src/intrinsic/mod.rs` (ledger constants) · `src/rete/purity.rs` (`KNOWN_UNREVIEWED` **only if**
a ratchet names it — the DESIGN predicts it will not). **No shared implementation changed. No
checker logic changed. No test deleted.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A wrapper will not compile for a reason the DESIGN did not anticipate. Report the
  exact error; do not reshape a shared implementation to fit the macro.
- **STOP-2.** The constructor arms' `split_type_param_bracket` pre-processing cannot be moved
  verbatim. Report it; do not simplify it — it decides what a `:- [T]`-spelled call evaluates.
- **STOP-3.** You cannot ground an axis, or a grounded reading says `@Purity Effectful`. Name the
  verb, the axis, what you read. **"I cannot tell" is the correct answer**, and it is worth more
  here than on the last two stones because no gate is checking your types.
- **STOP-4.** DEBT grows by anything other than exactly 5, or `KNOWN_UNREVIEWED` moves at all.
  Report the names.
- **STOP-5.** A test outside the ledger ratchets goes red. Copy that test's entire stdout and
  stderr block verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and
  report — before re-running anything.

## Verification, in this order

```bash
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

Read the Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
registry rows      537 → 542     +5 attribute sites, counted ANCHORED:
                                 grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                                   --include=*.rs | sort -u | wc -l
GAP_A               49 → 49      none of the five is on it
GAP_B               57 → 52      all five are on it
DEBT               106 → 111     ⬅ +5, all five. THE HONEST COST of this half of 1c.
                                 A different number means a different population registered.
KNOWN_UNREVIEWED    14 → 14      none of the five is on it — checked, not assumed
literal arms deleted  —  → 5
floor        5127/5127 → 5127/5127
clippy                    0
```

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the ground for each of the twenty-five axes (five verbs × five)
with the `file:line` you read, so each can be weighed — **especially the `@arg`/`@ret` types,
since no gate verifies them on this stone.**
