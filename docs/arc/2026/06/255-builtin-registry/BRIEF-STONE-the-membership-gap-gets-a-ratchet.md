# BRIEF — STONE: the membership gap gets a ratchet, and `fn`/`match` prove it moves

Three deliverables, one stone. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-membership-gap-gets-a-ratchet.md`.
The name list: `docs/arc/2026/06/255-builtin-registry/WORKLIST-the-121-the-registry-cannot-vouch-for.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5116, HEAD `a562b0575`.

## Read in order

1. The DESIGN, whole — especially § "Why a registration stone CANNOT go first" and the last
   acceptance row, which is a NEGATIVE this stone must preserve.
2. **`src/intrinsic/mod.rs`'s `checker_skip_debt_is_named_and_frozen`** — the bidirectional
   name-freeze both new gates copy. It is the working example; do not invent a second shape.
3. **`src/intrinsic/special/control_flow.rs`** — the `#[wat_special_form(":wat::core::if")]`
   doc-only unit struct. That file IS the template for deliverable 3; read its directive block
   before writing one.
4. `src/intrinsic/special/mod.rs` — how the sub-modules are declared and folded.

## The work

### 1 — `REGISTRY_MEMBERSHIP_GAP_A` and its gate (68 names)

The population is **derived, not listed**:

```
{ n : check_env.get(n).is_some()  ∧  registry().lookup_entry(n).is_none() }
```

Freeze the resulting names in a `const`, and gate bidirectionally exactly as
`checker_skip_debt_is_named_and_frozen` does — NEW (in the gap, not frozen) and STALE (frozen, no
longer in the gap), each naming the offending rows and saying what to do.

★ This is the builder's sentence made mechanical: *"the registry is not even the largest membership
set."* When the frozen list is empty, `registry ⊇ check_env`.

### 2 — `REGISTRY_MEMBERSHIP_GAP_B` and its gate (121 → 119 names)

The WORKLIST doc carries the names. Freeze them; assert per name that
`registry().lookup_entry(n).is_none()` still holds. A name that becomes a registry row fails as
**STALE** and must be deleted from the list — **that is the ratchet, and it is the point of the
stone.**

⚠ Gap B's population came from a corpus experiment a rider cannot run (it needs a build). The
WORKLIST doc records the four-step procedure; cite it in the gate's doc comment so the next reader
re-derives rather than trusts. Do not attempt the experiment.

### 3 — register `:wat::core::fn` and `:wat::core::match`

Two doc-only unit structs in `src/intrinsic/special/`, in the shape of `control_flow.rs`'s `If`.
Both already have named impls on **both** sides — annotate those existing functions with
`#[wat_special_form_impl(":wat::core::<name>", role = check)]` and `role = eval`:

```
:wat::core::fn      check → crate::function::infer_fn      eval → crate::function::eval_fn
:wat::core::match   check → infer_match (src/check.rs)     eval → eval_match_tail (src/runtime.rs)
```

⚠ **Verify each pairing against the dispatch arm before annotating.** The names above came from
reading arms; if the arm for a role calls something else, the arm is the truth and the brief is
wrong — report it.

Then **delete both names from `REGISTRY_MEMBERSHIP_GAP_B`** (121 → 119). If you do not, gate B fails
as STALE — which is the design working, not a bug.

Their directive blocks carry the real axes, derived from the forms' semantics the way `If`'s are:
`fn` constructs a closure and evaluates nothing; `match` evaluates its scrutinee and exactly one arm.
⚠ Both are `Preserving` on the operand-dependent axes if and only if you can ground that in one
sentence each — if you cannot, use `Unreviewed` and say so. **`Unreviewed` is the honest answer;
a guessed `Pure` is the lie the fourth variant exists to prevent.**

## Blast radius

`src/intrinsic/mod.rs` (two consts + two gates) · `src/intrinsic/special/` (two new sub-modules +
`mod.rs` declarations) · `src/function/` and `src/check.rs`/`src/runtime.rs` (four
`#[wat_special_form_impl]` annotations, no body changes) · whatever the compiler names. No `.wat`
corpus change. **No verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — DO NOT TOUCH `is_reserved_prefix` OR THE BLANKET-ACCEPT.** `src/resolve/walk.rs`'s
`if is_reserved_prefix(head) { return true; }` STAYS exactly as it is. Flipping it fails 578 of 599
corpus files — measured. It is the LAST stone of this thread, not this one.
`grep -c "if is_reserved_prefix(head)" src/resolve/walk.rs` must still be **1**.

**⛔ STOP-2 — `(:wat::holon::Bogus 1 2)` MUST STILL TYPE-CHECK CLEAN AFTER THIS STONE.** That is an
acceptance row, not an oversight. A stone that quietly closes the hole is a stone whose cascade
nobody measured.

**⛔ STOP-3 — NEITHER GATE FREEZES A COUNT.** Freeze names. A count cannot tell "+1 new, −1 fixed"
from "nothing happened", and its message cannot name the offender. ⚠ This campaign proved the rule
applies to the floor's own total: a test was silently disarmed while the floor read 5114 on both
sides, because a new test replaced it one-for-one.

**⛔ STOP-4 — GAP A IS DERIVED, NOT TRANSCRIBED.** Its population must be computed from
`check_env` × `registry()` at test time. If you hand-write the 68 names as the source of truth
instead of as the frozen comparison list, the gate stops measuring anything.

**⛔ STOP-5 — A GUESSED AXIS IS A LIE.** If you cannot ground `fn`'s or `match`'s `@Purity` /
`@Determinism` / `@Totality` / `@ExpandTime` in one sentence from the form's own semantics, write
`Unreviewed` and say why in the report. Do not copy `If`'s block across.

**STOP-6 — verbatim impls.** The four annotated functions keep their bodies exactly. This stone adds
attributes and rows; it moves no code.

**STOP-7 — your gates must be able to FAIL.** State, per gate and per direction, the exact edit that
would trip it and what the message would say. ⚠ You cannot execute the sabotage — you are forbidden
cargo — so **report it as unverified reasoning, explicitly, the way the previous rider correctly
did.** The orchestrator runs all four. Claiming a confirmation you could not perform is the failure;
naming the limit is the discipline.

## Report

Per-file diff summary; both gates' code verbatim; the two directive blocks verbatim with **the
grounding sentence for each axis** (or the `Unreviewed` and its reason); the four
`#[wat_special_form_impl]` pairings **and the dispatch arm you verified each against**; confirmation
that Gap B is now 119 and that STOP-1's grep is still 1; your STOP-7 sabotage reasoning per gate per
direction. Then: **what surprised you** — an axis you could not ground, an impl pairing the brief got
wrong, or a name in Gap A that looks like it should already be a row.
