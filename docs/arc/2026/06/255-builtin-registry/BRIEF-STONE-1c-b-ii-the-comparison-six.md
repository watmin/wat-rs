# BRIEF — STONE 1c-b-ii: the comparison six enter the registry

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-b-ii-the-comparison-six.md`

## The work, in one paragraph

Six `:wat::core::*` verbs — `=`, `not=`, `<`, `>`, `<=`, `>=` — have a literal dispatch arm and a
literal checker arm but **no `CheckEnv` scheme and no registry row**. None can be annotated in
place. **For each: write a thin wrapper fn, annotate it `#[wat_intrinsic("<fqdn>")]` with an
argued doc block carrying all five closed-domain axes, delete the literal arm, and apply what the
ledger ratchets name.** 812 corpus call sites — and `=`/`not=` unblock eight rete rows that have
been waiting since Phase 1b.

```
rt:2672  ":wat::core::="    => eval_eq(head, args, list_span, env, sym)
rt:2673  ":wat::core::not=" => eval_not_eq(head, …)
rt:2674  ":wat::core::<"    => eval_compare(head, …, |o| o == Ordering::Less)
rt:2677  ":wat::core::>"    => eval_compare(head, …, |o| o == Ordering::Greater)
rt:2680  ":wat::core::<="   => eval_compare(head, …, |o| o != Ordering::Greater)
rt:2683  ":wat::core::>="   => eval_compare(head, …, |o| o != Ordering::Less)
```

## ⛔ Why wrappers, and what each must carry

All three handlers take `head` as their **first** parameter, which is not the canonical
`#[wat_intrinsic]` shape; and `eval_compare` additionally takes a **predicate closure**. So each
of the six gets its own wrapper with the canonical signature `(args: &[WatAST], list_span: &Span,
env: &Environment, sym: &SymbolTable) -> Result<Value, EvalBreak>`, passing its own FQDN as `head`
— and **the four ordering wrappers must each carry their arm's closure verbatim.** Getting a
closure wrong swaps `<` for `<=`, and nothing in the type system would notice.

⛔ **The shared implementations are NOT touched**: `eval_eq`, `eval_not_eq`, `eval_compare`,
`infer_equality`, `infer_ordering` keep their current bodies and signatures.

## Read in order

1. **`src/runtime.rs`**, the five wrappers landed at Stone 1c-b-i (`eval_first`, `eval_second`,
   `eval_third`, `eval_persistentvector`, `eval_persistentmap`) — the exact shape you want.
2. **The three handlers** — `eval_eq` `:5221`, `eval_not_eq` `:5267`, `eval_compare` `:5681`.
3. **`src/check.rs:3797`** (`=`/`not=` → `infer_equality`) and **`:3813`** (the four orderings →
   `infer_ordering`), both inside `infer_list`. These are where the types come from.

⛔ **`src/check.rs:2423` carries the same two FQDNs and is NOT yours.** It lives inside
`infer_rete_form` — the rete-side routing for `:wat::rete::core::enum::=`. Two sites, two slots,
one out of scope. Do not read your types from it and do not touch it.

## ★★★ The one hard axis: what is `=` total ON?

Both runtime handlers carry an arity guard **and a real `TypeMismatch` raise** — `eval_eq` on a
`None` from its equality attempt, `eval_compare` on a non-orderable operand. An arity guard is
outside totality's domain (the established carve-out). **The `TypeMismatch` is the question, and
the question is not "does the runtime raise" but "can a WELL-TYPED call reach that raise."**

Read both checker helpers and answer it separately for equality and for ordering — they are
different fns and may differ:

- `infer_ordering`'s own comment says the orderings *"unify the two args (strict same-type, no
  subtype path), then gate on the orderable class"* — which would make the raise
  checker-unreachable, the same carve-out `:wat::core::get` was graded `Total` on this campaign.
- `infer_equality` is a separate helper. **It may be more permissive. Read it and say what you
  found.**

⚠ The precedent for the other answer is also on the record: Stone 1c-a-ii graded `conforms?`
**`Partial`** because its checker arm validated only syntax and never resolved the type, so a
well-typed call could still raise. Either grade is defensible; **only an ungrounded one is not.**

## The other four axes

Ground each from code you read, naming the fn or line. `src/collection/transform.rs` and the
1c-b-i wrappers show the register.

- ⚠ No gate checks your `@arg`/`@ret` here — `doc_arg_ret_types_match_checker_scheme` skips rows
  with no `CheckEnv` scheme, and none of these six has one. Ground every type in
  `infer_equality`/`infer_ordering` and report the `file:line` you read for each.
- `@ExpandTime`: authority is `src/macros/eval.rs`. ⚠ Note that Stone 1c-c **just deleted 52 dead
  rows** from that residue, so read it as it is now, not as any doc describes it. A name in
  neither the registry consult nor the residue is being silently refused with no ruling ever
  made — this arc has found four; if you find a fifth, say so, and grade what the verb IS rather
  than what today's accident produces.
- ⚠ `declared_purity_vs_effectful_by_prefix_census` requires an `@Purity Effectful` row's
  namespace to be in `effectful_by_prefix`; `:wat::core::` is not. A grounded `Effectful` reading
  is **STOP-3**, not something to grade around.

`@Totality Unreviewed` is not available: `KNOWN_UNREVIEWED` must not grow.

## Blast radius

`src/runtime.rs` (six wrappers + six doc blocks + six arm retirements) · `src/intrinsic/mod.rs`
(ledger constants). **No shared implementation changed. No checker logic changed. No test
touched. Nothing in `src/rete/`.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A wrapper will not compile for a reason the DESIGN did not anticipate. Report the
  exact error; do not reshape a shared implementation to fit the macro.
- **STOP-2.** A closure cannot be moved verbatim, or you are unsure which arm's closure belongs to
  which FQDN. **Report rather than guess** — a swapped closure silently inverts an operator.
- **STOP-3.** You cannot ground an axis, or a grounded reading says `@Purity Effectful`. Name the
  verb, the axis, what you read. "I cannot tell" is the correct answer.
- **STOP-4.** DEBT grows by anything other than exactly 6, or `KNOWN_UNREVIEWED`/`GAP_A` move at
  all. Report the names.
- **STOP-5.** A test outside the ledger ratchets goes red. Copy its entire stdout and stderr block
  verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and report — before
  re-running anything.

## Verification, in this order

```bash
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

★ Because a swapped closure is invisible to the type system, **also state in your report how you
verified each ordering wrapper carries the right predicate** — the floor's own comparison tests
are the natural witness; say which ones you saw pass.

## Acceptance — derived, not estimated

```
registry rows      542 → 548     +6 attribute sites, counted ANCHORED:
                                 grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                                   --include=*.rs | sort -u | wc -l
GAP_A               49 → 49      none of the six is on it
GAP_B               52 → 46      all six are on it
DEBT               111 → 117     ⬅ +6, all six. The honest cost.
KNOWN_UNREVIEWED    14 → 14      none of the six is on it — checked against the constant
literal arms deleted  —  → 6
floor        5128/5128 → 5128/5128   registering a row mints no `#[test]` fn
clippy                    0
```

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the ground for each of the thirty axes (six verbs × five) with
the `file:line` you read — **and separately the `file:line` for each `@arg`/`@ret`, since no gate
verifies those here.**
