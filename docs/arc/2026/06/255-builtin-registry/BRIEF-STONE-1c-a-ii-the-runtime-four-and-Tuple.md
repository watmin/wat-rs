# BRIEF — STONE 1c-a-ii: five more `:wat::core::` verbs, and Phase 1c-a closes

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-a-ii-the-runtime-four-and-Tuple.md`

## The work, in one paragraph

Five `:wat::core::*` verbs have a `CheckEnv` scheme and a literal dispatch arm but no registry
row. Register each as a `#[wat_intrinsic]` row with an argued `///` doc block carrying all five
closed-domain axes, delete its literal arm from the eval door, and apply what the ledger ratchets
name. Four are straightforward; **three carry a complication the DESIGN names in advance** — read
it before you start.

```
:wat::core::get         169 corpus sites   eval_get         (runtime.rs:7843)  canonical shape
:wat::core::contains?     7                eval_contains    (runtime.rs:7695)  canonical shape
:wat::core::conforms?     1                eval_conforms    (runtime.rs:7961)  canonical shape
:wat::core::apply        26                eval_apply       (runtime.rs:4985)  ⚠ needs a delegate
:wat::core::Tuple       271                INLINE at rt:2752                   ⚠ needs extraction
```

## Read in order

1. **`src/collection/transform.rs`** — six rows landed there last stone in exactly the shape you
   want: `#[wat_intrinsic]` on the handler in place, argued axes, no body change. Copy it.
2. **The five handlers**, at the `file:line`s above. Verify each signature before annotating.
3. **`src/runtime.rs`, `dispatch_keyword_head_value`** — the five arms to retire: `Tuple` at
   `:2752`, `apply` at `:2097`, `contains?` at `:2115`, `get` at `:2120`, `conforms?` at `:2129`.
   Line numbers are for finding, not for trusting.
4. **`src/intrinsic/mod.rs:2254`**, `doc_arg_ret_types_match_checker_scheme` — all five have a
   scheme, so this gate compares your `@arg`/`@ret` strings against it and reds with both
   spellings side by side. **Let it teach you the types; do not guess or reason one out.**

## ⚠ The three complications

**① `:wat::core::Tuple` — the one genuine extraction.** Its 18-line arm guards a documented
check-says-no / runtime-says-yes divergence: a declared-but-unpopulated `(Tuple :- [A B])` is an
arity mismatch the checker rejects, and answering it with an empty tuple here would re-create the
exact defect class the arm's own comment names. **Move the whole arm body verbatim into a named
fn and annotate that** — the `inner.is_empty() && rest.is_empty()` guard, the `split_type_param_bracket`
call, the two `eval_tuple_ctor` paths, and the comment explaining why, all unchanged. This stone
changes no behaviour.

**② `eval_apply` takes `list_span: Span` BY VALUE**, not `&Span`. `#[wat_intrinsic]` sniffs its
context tail as `&Environment` / `&SymbolTable` / `&Span` and rejects anything else at expand
time. So write a thin delegate with the canonical signature that forwards to the untouched
`eval_apply` — the same move `src/intrinsic/special/stream_lazy.rs` and `quote.rs` already make.
**Do not reshape the live fn.**

**③ Registering `apply` empties a positive control, and its disposition is already written.**
`tests/diagnostics/probe_substrate_symmetry_list_span_threading.rs`'s `MUST_FIND` has exactly one
anchor left — `:wat::core::apply`. Its own comment rules:

> *"When the last one goes, this positive control has nothing left to anchor on and should be
> DELETED rather than re-anchored."*

**Delete the `MUST_FIND` const and the `for` loop that consumes it — and NOTHING ELSE in that
file.** ⛔ The function `every_dispatch_arm_calling_eval_threads_list_span` stays: everything below
that loop (`classify_arm` over every arm, panicking on any arm that calls `eval` without threading
`list_span`) is the test's real job and is untouched by this stone. Leave a short comment where
the const was, naming this stone and quoting the header's ruling, so the retirement is legible.

⚠ Do **not** touch the `arms.len()` assertion. Its magnitude was retired last stone; it now reads
`!arms.is_empty()` and your five deletions cannot trip it.

## The five axes — argue each, from the code

For each verb state the **ground** for `@Purity`, `@Determinism`, `@Totality`, `@ExpandTime`,
`@Category`: name the fn you read, say what it does or does not call, let that decide the pole.
Worth knowing before you choose:

- `apply` invokes a caller-supplied function by keyword head — read what it does with effects and
  whether its own totality is its argument's. `Preserving` exists for that shape; read
  `and_form.rs`'s and `control_flow.rs`'s own `Preserving` arguments before reusing the word.
- `get` / `contains?` are container readers — read their real error paths. An arity guard is
  outside totality's domain (existing rows state that carve-out); a fallible lookup is not.
- `Tuple` is a constructor and type-directed. Read `split_type_param_bracket` and say what it
  actually evaluates.
- `@ExpandTime`: the authority is `src/macros/eval.rs` — `is_expand_time_legal`'s registry-first
  consult and its residue list. Grep each FQDN there. ⚠ A name in NEITHER is being silently
  refused today; Stone 1a-ζ found exactly that for `ann-form` and it was a real gap. Say so if
  you find one.
- ⚠ A structural constraint that has already refuted one draft: `declared_purity_vs_effectful_by_prefix_census`
  requires an `@Purity Effectful` row's namespace to appear in `effectful_by_prefix`. `:wat::core::`
  is NOT in that list, so an `Effectful` grade here will red. If your reading says `Effectful`,
  that is **STOP-3** — report it rather than grading around the gate.

`@Totality Unreviewed` is not available: it must not grow.

## Blast radius

`src/runtime.rs` (four annotations, one extraction, one delegate, five arm retirements; **no
handler body changed**) · `src/intrinsic/mod.rs` (ledger constants) · `src/rete/purity.rs`
(`KNOWN_UNREVIEWED` only, if the ratchet names it) · `tests/diagnostics/probe_substrate_symmetry_list_span_threading.rs`
(the `MUST_FIND` const and its loop, nothing else). Nothing else.

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A handler's signature will not take `#[wat_intrinsic]` for a reason the DESIGN did
  not anticipate. Report which and how; do not reshape a live handler to fit the macro.
- **STOP-2.** Extracting `Tuple`'s arm cannot be done verbatim — the body references something
  not reachable from a free fn. Report it; do not rewrite the guard.
- **STOP-3.** You cannot ground an axis, or your grounded reading is `@Purity Effectful`. Say
  which verb, which axis, what you read. **"I cannot tell" is the correct answer.**
- **STOP-4.** `FROZEN_CHECKER_DEBT_LEDGER` gains any name. The DESIGN derives it unchanged at 106.
- **STOP-5.** A test outside the ledger ratchets and the named `MUST_FIND` edit goes red. Copy
  that test's entire stdout and stderr block verbatim from `.floor/latest/raw.log`, name the exact
  assertion that fired, and report — before re-running anything.

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
registry rows      532 → 537     +5 attribute sites, counted ANCHORED:
                                 grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                                   --include=*.rs | sort -u | wc -l
GAP_A               54 → 49      all five are on it
GAP_B               62 → 57      all five are on it
DEBT               106 → 106     ⬅ UNCHANGED — the row that cannot be faked
KNOWN_UNREVIEWED    17 → 14      apply · conforms? · Tuple are on it — PREDICTED, not a surprise
literal arms deleted  —  → 5
MUST_FIND            1 → 0       the const and its loop retire; the TEST DOES NOT
floor        5127/5127 → 5127/5127
clippy                    0
```

★ On landing, **Phase 1c-a is complete**: all eleven `:wat::core::` names that already had schemes
are registered.

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the ground for each of the twenty-five axes (five verbs × five)
with the fn you read, so each can be weighed.
