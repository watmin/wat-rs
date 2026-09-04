# BRIEF — STONE 1c-d: `extend-type` · `derive` · `defclause` enter the registry

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-d-the-declaration-three.md`

## The work, in one paragraph

Three `:wat::core::*` declaration forms have a declare-time parser and a checker arm but **no
dispatch arm, no registry row, and no `special_forms.rs` entry**. Give each a doc-only
`#[wat_special_form]` struct with an argued `///` block, then annotate its two existing functions
with `#[wat_special_form_impl(…, role = declare)]` and `#[wat_special_form_impl(…, role = check)]`.
**Nothing is extracted and nothing is rewritten** — this is wiring two pointers per row onto code
that already exists. 212 corpus call sites.

```
:wat::core::extend-type  157 sites  parse_extend_type_form  src/function/parse.rs:937   check.rs:2647
:wat::core::derive        47        parse_derive_form       src/function/parse.rs:1273  check.rs:2666
:wat::core::defclause      8        parse_defclause_form    src/function/parse.rs:690   check.rs:2635
```

## Read in order

1. **`src/intrinsic/special/def.rs`** (or `defmacro.rs` / `defalias.rs`) — a row in exactly this
   regime: `@Purity Unevaluated`, a `role = declare` pointer, no eval impl. Copy it.
2. **`src/intrinsic/special/use_form.rs`** — the closest *argued* precedent, and read its
   `@Category` ground carefully: it walks through why `Declaration` fits and why `Ambient` and
   `CheckGate` do not. That is the shape of reasoning each of your three needs.
3. **The three declare-time fns** at the `file:line`s above, and **the three checker arms**.
   Verify each before annotating; line numbers are for finding, not for trusting.
4. **`src/intrinsic/mod.rs`**, `every_special_form_carries_check_and_eval_impls` — the gate that
   makes `@Purity Unevaluated` the only admissible grade here. Read it so the grade is understood
   rather than copied.

## ★★★ Registering `defclause` restores a refusal that was lost, by construction

`SEAM:214` has carried this since the hand-rolled-arms stone: *"`defclause` lost its named refusal
this session; it has no registry row. Register it."*

That stone retired the hand-rolled `def`/`defclause` arms in favour of the registry-first door's
`Unevaluated` guard. `def` had a row and kept a named refusal; **`defclause` had none and lost
one** — `src/runtime.rs:2054-2058` and `:2250` record the loss at the site.

The moment `defclause`'s row exists with `@Purity Unevaluated`, `dispatch_keyword_head`'s guard
answers `DeclarationInExpressionPosition` for it again. **Verify that directly** — write a probe
that puts `(:wat::core::defclause …)` in expression position and confirm the named error comes
back — and report what you saw. A refusal restored by construction is this stone's most
interesting result, and it is worth more than the three rows.

## The axes

`@Purity Unevaluated` is forced by the gate and true by construction: all three are consumed by
the freeze pipeline before evaluation begins, and none has an eval impl to name. State the ground
anyway, from the fn you read.

⚠ **`@Category` is the one to argue, not assume.** `:Declaration`'s prose — *"registers a
program-level entity … visible to everything after it"* — fits all three on its face. But Stone
1a-δ proved that reading wrong: the loaders looked like declarations and turned out to be
`:Splice`, because a load registers nothing, it replaces itself with N forms. **Read what each of
your three actually does to the program.** `extend-type` and `derive` both touch
`src/types.rs` (`:3918`, `:3886`) as well as the parser — follow that and say what it means.

For `@Determinism`, `@Totality`, `@ExpandTime`: ground each from the declare-time fn.
`@Totality Unreviewed` is not available; `KNOWN_UNREVIEWED` must not grow (it should SHRINK by
one — `derive` is on it).

## Blast radius

`src/intrinsic/special/` (three new files + three `mod` lines) · `src/function/parse.rs` and
`src/check.rs` (annotations only — **no body changes**) · `src/intrinsic/mod.rs` and
`src/rete/purity.rs` (ledgers, per the ratchets). **No arm is deleted — there are none.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A `role = declare` or `role = check` annotation will not compile. Report the exact
  error; do not reshape a live parser fn to fit the macro.
- **STOP-2.** The registration gate refuses one of the three — e.g. it demands an eval impl.
  That means the row is not `Unevaluated` in the gate's eyes and the DESIGN's reading is wrong.
  Report rather than inventing an eval delegate.
- **STOP-3.** Your grounded `@Category` reading is NOT `Declaration` for one of the three. That is
  a finding, not a problem — say which, and what it actually does. `:Splice` was found exactly
  this way.
- **STOP-4.** `defclause` in expression position does NOT produce the named refusal after
  registration. Report what it produces instead — the DESIGN's central claim would then be wrong.
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

Read the Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
registry rows      546 → 549     +3 attribute sites, counted ANCHORED:
                                 grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                                   --include=*.rs | sort -u | wc -l
GAP_A               49 → 49      none of the three is on it
GAP_B               48 → 45      all three are on it
DEBT               115 → 118     +3, no CheckEnv scheme exists for any of them
KNOWN_UNREVIEWED    14 → 13      ⬅ −1: `derive` IS on it; the other two are NOT. Checked
                                 against the constant, not assumed.
literal arms deleted  —  → 0     ⬅ THERE ARE NONE. The no-literal-arm gate has nothing to
                                 demand here, which is the tell that this is a third regime.
defclause refusal          RESTORED by the Unevaluated guard — verify and report
floor        5129/5129 → 5129/5129
clippy                    0
```

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the ground for each of the fifteen axes (three verbs × five)
with the fn or `file:line` you read, and report the `defclause` refusal probe in full.
