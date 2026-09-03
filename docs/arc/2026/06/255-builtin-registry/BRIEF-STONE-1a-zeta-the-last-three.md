# BRIEF — STONE 1a-ζ: `do` · `ann-form` · `stream::lazy` enter the registry, and Phase 1a ends

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-zeta-the-last-three-of-the-special-form-table.md`

## The work, in one paragraph

Three rows of `src/special_forms.rs` have never been registered in the intrinsic registry:
`:wat::core::do`, `:wat::core::ann-form`, `:wat::stream::lazy`. Give each a doc-only
`#[wat_special_form]` struct with a full argued doc block — prose, `@added`, its shape, `@ret`,
`@example`, **and all five closed-domain axes, each with its ground stated from the code** — then
annotate its existing implementation fns with `#[wat_special_form_impl(…, role = …)]` so the
registry holds the pointers. These are **not** aliases: each is a real special form with its own
behaviour, so each argues its own five axes. After this stone, every row in `special_forms.rs`
that a stone can register is registered, and Phase 1a is complete.

## Read in order

1. **`src/intrinsic/special/quasiquote.rs`** — the template for an argued (non-alias) special-form
   row: how a `///` block states each axis's **ground** ("measured directly: `eval_forms` never
   calls `eval` on any of `args`…"), and how `#[wat_special_form_impl(…, role = check)]` wires a
   named fn. Copy this shape, not an alias row's.
2. **`src/intrinsic/special/control_flow.rs`** (`:wat::core::if`) and
   **`src/intrinsic/special/and_form.rs`** — the two closest precedents for a row that carries
   **three** roles (check, eval, tail) and for `Preserving`-style axis arguments on a form whose
   sub-forms are not all evaluated.
3. **`src/special_forms.rs`** lines 142 (`ann-form`), 165 (`do`), 244 (`stream::lazy`) — the
   declared arity sketch each row must stay consistent with.
4. **The implementation fns you will annotate:**
   ```
   :wat::core::do          eval  src/runtime.rs:2260 -> eval_do        tail rt:998  -> eval_do_tail
                           check src/check.rs:2881
   :wat::core::ann-form    eval  src/runtime.rs:2267 -> eval_ann_form  tail rt:1009 -> eval_ann_form_tail
                           check src/check.rs:3238
   :wat::stream::lazy      eval  src/runtime.rs:2279 -> eval_lazy_seq  (NO tail arm — no role = tail)
                           check src/check.rs:3304
   ```
   Verify each at its line before annotating; the line numbers are for finding, not for trusting.
5. **`src/intrinsic/mod.rs:2999`**, `registry_first_door_owns_every_handler_row_no_literal_arm_survives`
   — read this gate before you start. It is what makes the next section non-negotiable.

## ⛔ THE THREE EVAL ARMS MUST BE DELETED — expect it, do not be surprised by it

The gate above filters on `entry.handler.is_some()`. The moment a row carries `role = eval` it
has a handler, and the gate **requires that its literal arm inside `dispatch_keyword_head_value`
be gone** — `runtime.rs:2260`, `:2267`, `:2279`. The registry-first door at the top of that
function answers those heads instead; nothing is lost.

Follow the retirement convention already in that function: delete the arm and leave a short
comment in its place saying which stone retired it and where the row now lives, exactly as the
neighbouring retired arms do. **The handler fns themselves (`eval_do`, `eval_ann_form`,
`eval_lazy_seq`) stay** — they are what `role = eval` points at.

The **tail** arms are different and are NOT covered by that gate (its span is bounded to
`dispatch_keyword_head_value`, and `tail_handler` is a separate field consulted only by
`eval_tail`'s own guard). Follow the `if`/`let`/`match`/`and`/`or` precedent visible around
`runtime.rs:976–1010`: annotate `role = tail`, then retire the arm the same way, with the same
kind of comment. `stream::lazy` has no tail arm and must not be given a `role = tail`.

## The five axes — argue each, from the code

For each of the three rows, state the **ground** for `@Purity`, `@Determinism`, `@Totality`,
`@ExpandTime`, `@Category`, the way `quasiquote.rs` and `and_form.rs` do: name the fn you read,
say what it does or does not call, and let that decide the pole. Things worth reading before you
choose:

- `do` evaluates every sub-form in sequence and returns the last — so its axes plausibly
  *follow* its sub-forms rather than being absolute. `Preserving` exists for exactly that shape
  (`control_flow.rs`'s `if` argues it; read that argument before reusing the word).
- `ann-form` is a type ascription. Read `eval_ann_form` and the check arm at `check.rs:3238` and
  say what it actually does at runtime versus at check time — the two are not the same claim.
- `stream::lazy` is "capture-don't-eval" (`runtime.rs:6115`'s own comment). A form whose body is
  *not* evaluated when the form is has a different purity story from one whose body is.
- `@ExpandTime`: the authority is `src/macros/eval.rs` — `is_expand_time_legal`'s allow-list and
  `validate_pure_total`. Grep for each FQDN there and let what you find decide, rather than
  reasoning from the verb's nature.

⚠ `@Totality Unreviewed` is not available to you: `KNOWN_UNREVIEWED` must not grow. If you
genuinely cannot ground a pole from the code, that is **STOP-3** — report it; do not pick the
comfortable pole.

## Blast radius

`src/intrinsic/special/` (three new files, one per row, following the directory's one-row-per-file
convention) · `src/intrinsic/special/mod.rs` (three `pub(crate) mod` lines) · `src/runtime.rs`
(annotations + arm retirements) · `src/check.rs` (annotations; the check fns' own logic is
untouched — if an arm's body must be lifted into a named fn to be annotated, move it **verbatim**)
· `src/intrinsic/mod.rs` (ledger constants only). **`src/special_forms.rs` keeps all 30 rows.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A `role = eval` or `role = tail` annotation will not compile because a shim of that
  name already exists. `role = eval`/`tail` shims are keyed on the **fn identifier**, not the
  FQDN, so two rows pointing at one fn collide. Report the collision; do not rename a live fn to
  dodge it.
- **STOP-2.** Deleting an eval arm changes behaviour for some other head (e.g. the arm was part of
  an OR-pattern serving several names). Report which; do not split it on your own judgement.
- **STOP-3.** You cannot ground one of the five axes from the code. Say which row, which axis,
  what you read, and what you could not determine. **"I cannot tell" is the correct answer here**
  and is worth far more than a plausible pole.
- **STOP-4.** `KNOWN_UNREVIEWED` (`src/rete/purity.rs`) grows, or `GAP_A` moves at all. Neither
  should. Report the names.
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

Expect two rounds of red, both informative: first the no-literal-arm gate demanding the three
arms come out, then the ledger ratchets naming their edits. Both are the floor telling you the
next step. Read the Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
registry rows      523 → 526     +3 attribute sites, counted ANCHORED:
                                 grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                                   --include=*.rs | sort -u | wc -l
GAP_A               60 → 60      none of the three has a scheme, so none is on GAP_A
GAP_B               71 → 68      all three are on GAP_B (verified against the constant)
DEBT               103 → 106     +3 — each has a literal check ARM, never a registered scheme
KNOWN_UNREVIEWED    20 → 20      each row argues its own Totality
floor        5127/5127 → 5127/5127   registering a row mints no `#[test]` fn
clippy                    0
```

## Working rules

Everything foreground. You may not spawn sub-agents. No worktrees, no `git stash`, no
`git revert`, no commit, no push — leave the tree dirty and report; the orchestrator commits.
This stone's central content is five arguments per row that no gate can check, so report the
ground you found for each axis, with the fn you read, so it can be weighed.

Shape to copy: `src/intrinsic/special/quasiquote.rs`, and `BRIEF-STONE-1a-gamma-i-the-homoiconic-six.md`.
