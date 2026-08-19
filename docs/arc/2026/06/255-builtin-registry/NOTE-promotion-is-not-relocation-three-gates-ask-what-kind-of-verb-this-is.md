# ⛔ NOTE (arc 255) — promoting a verb to an intrinsic is NOT a relocation. It changes what can CALL it, what AUDITS it, and how it TYPE-CHECKS.

**Written 2026-08-18 from arc 118.B4-0 (`8f5252a0`), which promoted `:wat::core::nth` from a wat
`defclause` to a Rust intrinsic and hit all three effects in one strike.** Recorded here rather than
in 118 because the cause is 255's and so is the fix.

## What happened, in one paragraph

Arc 118 needed `nth` callable from a `defmacro` program body. It was not — not because it was
refused, but because a macro body evaluates through `dispatch_keyword_head`, and `nth` was wat. Making
it native took **five `src/` files**, of which the brief predicted four; the fifth was a purity ledger
nobody involved knew was watching. And the promotion silently dropped a type-checking capability the
`defclause` form had been providing for free, breaking a pre-existing test in a distant file.

## Three separate hand-maintained gates asked "what kind of verb is this?" — none could answer from one place

| gate | site | what it wanted | how it was satisfied |
|---|---|---|---|
| macro-body reachability | `dispatch_keyword_head`, `src/runtime.rs` | is this verb dispatched at all? | by *becoming* an intrinsic |
| expand-time purity | `is_pure_total`, `src/macros/eval.rs` | is it pure, among the dispatched? | a hand-added entry |
| ledger completeness | `rete::purity::completeness_gate` → `intrinsic_meta` | does every dispatched verb carry a ruling? | a second, independent hand-added entry |

**The second and third are different concerns with no link between them.** Satisfying `is_pure_total`
did nothing for the completeness gate; the floor went red on the latter after the former was already
green. Two hand-lists, two rulings, one verb, and the only thing that discovered the gap was a test
run. `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`

★ **This is 255's thesis stated as a bug report.** All three questions are metadata questions —
*is it dispatched · is it pure · is it total* — and the registry exists to answer exactly those from
one source. Today they were answered three times, by hand, in three files, by three different people's
conventions.

## ⛔ THE PART THE CARVE WILL MEET REPEATEDLY — a clause unifies; an arm classifies

`255`'s own accounting (`NOTE-2026-08-14-regrounding-the-premise.md`) counts **141 hand-written
inference arms inside `infer_list`**, and names shrinking them as the work. **Today made it 142** —
and proved that the count understates the cost.

A multi-arm `defclause` type-checks by **UNIFYING** the receiver against each arm's declared parameter
type. A free type variable therefore binds as a side effect of dispatch: the checker tries
`Vector<T>`, `PersistentVector<T>`, `List<T>`, `Seqable<T>` in turn, and whichever unifies resolves
the variable.

A hand-written `infer_list` arm does not unify. It **CLASSIFIES** — `StreamContainer::of_type(&reduced)`
on an already-reduced `TypeExpr` — and a `TypeExpr::Var(_)` classifies as `None`, so it errors where
the clause would have succeeded.

**The failure it produced**, in a file the strike never touched:

```wat
(nth (PersistentVector/conj (PersistentVector) 7) 0)
```

An empty PersistentVector, then `conj` — the element type is unresolved until unification lands.
Green under the `defclause`. Hard `TypeMismatch` under the first draft of `infer_nth`. The fix is an
explicit defer-to-runtime branch on `TypeExpr::Var(_)` **before** the classification dispatch,
mirroring `infer_get`'s existing convention.

**So 141 arms are not 141 units of duplication. They are 141 places where free-variable deferral can
be forgotten, and forgetting is invisible in the common path** — it surfaces only when a caller
constructs a value whose element type is not yet resolved (empty-then-`conj`, an empty literal, a
generic helper's return threaded through). A carve that passes all of its own new tests can still
break a distant pre-existing one.

## What this asks of 255 — two things, neither of them "remember"

1. **A carve checklist row that is mechanical, not remembered:** does this verb's inference arm have a
   `Var`-defer branch? Every promotion needs one; the compiler will not ask.
2. **Consider whether the registry can make the defer STRUCTURAL.** If a verb's accepted receiver set
   is data, the "unresolved receiver ⇒ defer to runtime" rule can be derived once rather than
   hand-written 142 times. That is the same *derive it, do not hand-maintain it* move the macro-body
   allow-list needs (task #107) — and the same shape as the `is_pure_total` / `intrinsic_meta` split
   above.

## What this note does NOT claim

- **It does not claim 255's carve is wat→native promotion.** 255 relocates existing Rust builtins into
  namespaced homes; today's strike was a different operation. The findings transfer because both cross
  the same boundary and consult the same gates — not because the operations are the same. Do not cite
  this note as evidence about the carve's shape.
- **It does not rule anything.** The `Var`-defer fix is landed for `nth` only. Whether the other 141
  arms have the same hole is **unmeasured** — and measuring it is one grep away, which is exactly why
  nobody should assume either answer.

## Kin

- `NOTE-an-intrinsic-cannot-be-passed-as-a-value.md` — the mirror: an intrinsic has *less* capability
  than a user fn in value position, where here it has *more* in call position. Same boundary, opposite
  direction. Whoever touches the wat/native line should read both.
- `NOTE-2026-08-14-regrounding-the-premise.md` — the 332 / 141 / 678 / 0 table this note updates.
- `NOTE-purity-is-definition-time-queryable-metadata.md` — the two purity gates above are the argument
  for that note, found in the wild.
- Tasks #107 (macro bodies reach only intrinsics), #108 (the unification loss).

---

> Found because a stone in another arc needed one verb to be callable one layer down, and the substrate
> charged it three separate tolls, each collected by a different hand-maintained list, one of which
> nobody knew existed until the floor went red.
