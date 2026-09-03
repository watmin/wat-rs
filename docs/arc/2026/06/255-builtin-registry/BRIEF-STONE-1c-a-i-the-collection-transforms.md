# BRIEF — STONE 1c-a-i: six collection transforms enter the registry

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-a-i-the-collection-transforms.md`

## The work, in one paragraph

Six `:wat::core::*` verbs have a `CheckEnv` scheme and a literal dispatch arm but no registry row.
All six are ordinary functions living in one module, `src/collection/transform.rs`, and all six
share one identical handler signature. **Annotate each handler fn in place with
`#[wat_intrinsic("<fqdn>")]`, give each an argued `///` doc block including all five closed-domain
axes, then delete its now-redundant literal arm from the eval door.** Finish by applying whatever
the ledger ratchets name.

```
:wat::core::foldl              380 corpus call sites   eval_vec_foldl
:wat::core::map                 65                     eval_vec_map
:wat::core::filter               9                     eval_filter
:wat::core::stream->vec          6                     eval_stream_to_vec
:wat::core::mapv                 2                     eval_mapv
:wat::core::find-last-index      1                     eval_vec_find_last_index
```

## Read in order

1. **`src/intrinsic/collection.rs`** — the closest precedent: `#[wat_intrinsic]` rows on ordinary
   collection verbs, with argued axes. Copy this shape (NOT a `#[wat_special_form]` row, and NOT
   an alias row — these are real functions with their own properties).
2. **`src/collection/transform.rs`** — your six handlers, at `eval_vec_foldl:596` ·
   `eval_vec_map:426` · `eval_filter:1260` · `eval_stream_to_vec:738` · `eval_mapv:473` ·
   `eval_vec_find_last_index:1075`. Verify each signature before annotating; all six should read
   `(args: &[WatAST], call_span: &Span, env: &Environment, sym: &SymbolTable) -> Result<Value, EvalBreak>`,
   which is `#[wat_intrinsic]`'s variadic shape with a context tail — **no delegate needed.**
3. **`src/runtime.rs`, `dispatch_keyword_head_value`** — the six arms to retire, at `:2860`
   (`foldl`) · `:2851` (`map`) · `:2867` (`filter`) · `:2558` (`stream->vec`) · `:2852` (`mapv`) ·
   `:2570` (`find-last-index`). Line numbers are for finding, not for trusting.
4. **`src/intrinsic/mod.rs:2999`**, `registry_first_door_owns_every_handler_row_no_literal_arm_survives`
   — read this before you start; it is what makes the arm deletions non-negotiable.
5. **`src/intrinsic/mod.rs:2254`**, `doc_arg_ret_types_match_checker_scheme` — read this too. All
   six already have a `CheckEnv` scheme, so this gate **actively compares your `@arg` and `@ret`
   strings against it** and reds with both spellings side by side. It is your teacher: author,
   run, read the failure, correct. Do not guess a type spelling.

## ⛔ The six arms MUST be deleted — expect it

`#[wat_intrinsic]` mints a dispatch shim, so each row gets `handler: Some`, and the gate in (4)
then requires its literal arm be gone. Follow the retirement convention already visible in that
function: delete the arm, leave a short comment in its place naming this stone and where the row
now lives, exactly as the neighbouring retired arms do. **The handler fns themselves stay** —
they are what the annotation registers.

## The five axes — argue each, from the code

For each of the six, state the **ground** for `@Purity`, `@Determinism`, `@Totality`,
`@ExpandTime`, `@Category`: name the fn you read, say what it does or does not call, and let that
decide the pole. `src/intrinsic/collection.rs`'s existing rows show the register.

Worth reading before choosing:
- These are **higher-order** — `foldl`, `map`, `filter`, `mapv` all invoke a caller-supplied
  function. A verb whose effects are its argument's effects is what `Preserving` was minted for;
  read `and_form.rs`'s and `control_flow.rs`'s own `Preserving` arguments before reusing the word,
  and say which applies.
- `@Totality`: read each handler's real error paths. An arity guard is outside totality's domain
  (the existing rows state that carve-out); a fallible element access is not.
- `@ExpandTime`: the authority is `src/macros/eval.rs` — `is_expand_time_legal`'s registry-first
  consult and its residue list. Grep each FQDN there and let what you find decide. ⚠ A name in
  NEITHER the registry nor the residue is being silently refused today; if you find that, say so
  — Stone 1a-ζ found exactly that for `ann-form` and it was a real gap.

⚠ `@Totality Unreviewed` is not available: `KNOWN_UNREVIEWED` must not grow. If a pole cannot be
grounded from the code, that is **STOP-3** — report it rather than picking the comfortable pole.

## Blast radius

`src/collection/transform.rs` (six annotations + six doc blocks; **no handler body changes**) ·
`src/runtime.rs` (six arm retirements) · `src/intrinsic/mod.rs` (ledger constants only). Nothing
else. No new module. No `.wat` change. No test deleted.

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A handler's signature is not the canonical shape and `#[wat_intrinsic]` will not
  take it. Report which and how it differs; do not reshape a live handler to fit the macro.
- **STOP-2.** Deleting an arm changes behaviour for another head — e.g. the arm was part of an
  OR-pattern serving several names. Report which; do not split it on your own judgement.
- **STOP-3.** You cannot ground one of the five axes from the code. Say which verb, which axis,
  what you read, what you could not determine. **"I cannot tell" is the correct answer here.**
- **STOP-4.** `FROZEN_CHECKER_DEBT_LEDGER` gains any name, or `KNOWN_UNREVIEWED` grows. The
  DESIGN derives DEBT unchanged at 106; a rise means a type was mis-transcribed or a row was
  registered that has no scheme. Report the names.
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

Expect two rounds of red, both informative: the no-literal-arm gate demanding the six arms come
out, then the ledger ratchets naming their edits. Both are the floor telling you the next step.
Read the Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
registry rows      526 → 532     +6 attribute sites, counted ANCHORED:
                                 grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                                   --include=*.rs | sort -u | wc -l
GAP_A               60 → 54      all six are on it (verified against the constant)
GAP_B               68 → 62      all six are on it
DEBT               106 → 106     ⬅ UNCHANGED — the row that cannot be faked
KNOWN_UNREVIEWED    18 → 18
literal arms deleted  —  → 6
floor        5127/5127 → 5127/5127   registering a row mints no `#[test]` fn
clippy                    0
```

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the ground you found for each of the thirty axes (six verbs ×
five), with the fn you read, so each can be weighed.

Shape to copy: `src/intrinsic/collection.rs`, and `BRIEF-STONE-1a-zeta-the-last-three.md`.
