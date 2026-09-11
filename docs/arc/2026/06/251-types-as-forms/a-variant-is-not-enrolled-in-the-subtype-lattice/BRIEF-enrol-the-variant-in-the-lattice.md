# BRIEF — enrol the variant in the subtype lattice (the edge nobody registered)

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. Never use worktrees.
Do not touch `~/work/holon/` (the frozen root).

## Tree state

HEAD `6587423e1`, **clean**, 30 commits UNPUSHED. `cargo clippy --release --all-targets -- -D warnings` → 0.

```
floor (.floor/2026-09-11T06-15-41Z)   5360 run   5354 passed   6 failed   22 skipped
```

**Those 6 ARE yours.** They are the whole of the remaining floor, they share one root cause, and
this stone is expected to take the floor to **0 failed**. Baseline captured at that log dir — the
zero point is 6, not 8, not 9.

## The builder's semantics — SETTLED, do not re-derive

> `user/Box.Full` is both a `user/Box` AND a `user/Box.Full`.
> A caller taking an explicit **variant** may only receive that variant.
> A caller taking the **enum** may be handed any variant, but must `match` on it.

`Variant <: Enum`. Widening free, narrowing refused. The same semantics as `match`.

## The defect — ONE MISSING EDGE, and the machinery is already wired to consume it

The failure, four identical `TypeMismatch`es across two fixtures:

```
:wat::core::foldl: parameter #1
  expects [ … (:wat::service::Alarm :- [:probe::callctx3svc::Op.-Mark]) … ]
  got     [ … (:wat::service::Alarm :- [:probe::callctx3svc::Op])        … ]
```

Born at `tests/services/probe_arc278_call_context.wat:84` —
`:arms [(:wat::service::Alarm :after (…) :op :-mark)]`. `Alarm` is a **defrecord**
(`wat/service.wat:56`, `:- [O] [after <- Duration  op <- :O]`), so `O` is inferred from the op
keyword at its most specific type, the variant singleton. (`Op.-Mark` is correct: a leading `-`
marks an INTERNAL service arm, so the generated `Op` enum carries a `-Mark` variant.)

### ★ The consumer already exists and already asks the right question

`src/check.rs:17406` — the SAME-head parametric arm of `assignable`:

```rust
if ah == eh && aargs.len() == eargs.len()
    && aargs.iter().zip(eargs.iter()).all(|(x, y)| {
        if unify(x, y, subst, types).is_ok() { return true; }
        let xr = reduce(&walk(x, subst), subst, types);
        let yr = reduce(&walk(y, subst), subst, types);
        if matches!(&xr, TypeExpr::Path(p) if p == ":wat::core::Never")
            || matches!(&yr, TypeExpr::Path(p) if p == ":wat::core::Value")
            || matches!((&xr, &yr), (TypeExpr::Path(xp), TypeExpr::Path(yp))
                if crate::types::is_subtype(xp, yp, types))   // <-- ASKED, answers false
        { return true; }
```

Its own doctrine, verbatim: *"SAME-head parametric: args are INVARIANT (unify) **EXCEPT that the
subtype LATTICE flows through each arg position** … Only a genuine subtype edge (endpoints, or a
user-declared parent) relaxes a slot."*

`Alarm<Op.-Mark>` vs `Alarm<Op>` lands here: same head, both args bare `Path`s. **`is_subtype` is
called and returns `false` only because no edge was ever registered.**

### Why the edge is absent

`src/types.rs:1069  register_variant_types` synthesizes one singleton `TypeDef::Enum` per variant
and **registers no subtype edge to the parent enum**. The parent is already computed:
`src/types.rs:1003  variant_parent_enum(name) -> Option<&str>`. The pusher already exists:
`src/types.rs:927  register_subtype(child, parent, span)` (with a cycle check).

`struct ← core-record ← holon-record` is this same table. Variants were simply never enrolled.

## The strike

In `register_variant_types`, for each synthesized variant singleton, register
`register_subtype(<composed variant name>, <parent enum name>, span)`.

Direction matters and is one-way: **child = variant, parent = enum.** In the arm above, `x` is the
ACTUAL and `y` the EXPECTED, so `is_subtype(actual, expected)` — variant→enum accepted,
enum→variant still refused with no edge. That is the builder's rule, structurally.

## ⛔ STOP triggers — each is a REJECTION, report and halt

**STOP-1 — `is_subtype_parent` reclassification.** `src/types.rs:956` decides "is `name` a
derive-marker / typesub parent" by scanning the **VALUES** of `subtype_edges`. Enrolling every
variant makes **every enum** appear as a value, so `is_subtype_parent(":…SomeEnum")` flips to
`true`. That predicate feeds `types.rs:641`, `src/reflect/verbs.rs:1528`, and
`src/declare/typevar.rs:234`. **Measure what that reclassifies BEFORE assuming it is harmless.**
If it changes any behaviour, STOP and report — the fix may need a separate edge map rather than a
wider meaning for an existing one.

**STOP-2 — the cycle check.** `register_subtype` refuses an edge that closes a cycle. A variant
whose parent resolves to itself (or a singleton re-walked as an enum) would trip it. If any
registration returns `CyclicSubtype`, STOP with the exact child/parent pair — that names a
name-composition bug, not a lattice bug.

**STOP-3 — stdlib scale.** This enrols EVERY variant of EVERY enum, `Option`/`Result` included.
If `is_subtype_parent`'s linear scan over all values shows up as a measurable slowdown in the
floor's wall time (baseline 205.5s), STOP and report the number.

**STOP-4 — if any currently-PASSING test goes red.** Especially
`probe_arc278_journal_surface::wrong_response_type_at_reply_site_is_compile_error`, whose golden
pins a NARROW variant name. Arc 296 A-2 RELAND-4 measured that over-eager widening corrupts a
bare-symbol binder's STORED type and changes that test's error text. If it moves, STOP — you have
widened something that must stay narrow.

## Acceptance

```
1. the 6 arc278 reds go GREEN
2. a NEGATIVE CONTROL, and it must be written and must FAIL before it passes:
     Demo.Has -> Demo      slot   ACCEPTED
     Demo     -> Demo.Has  slot   REFUSED      <-- prove this still refuses
     Demo.Has -> Demo.Has  slot   ACCEPTED
3. scripts/floor.sh          expect 5360 run / 5360 passed / 0 failed
4. clippy --release --all-targets -- -D warnings   expect 0
```

Row 2 is the stone. Without it a green floor proves only that you loosened something, not that you
loosened exactly the right thing — and the builder's rule is a PAIR of claims, one of which is a
refusal.

## Tier

You edit, you write the negative control, and you REPORT. **Do NOT commit. Do NOT run
`scripts/floor.sh` or clippy** — the orchestrator measures centrally, once, on a quiescent tree.

⛔ Do NOT call `pulsare_yield` or contact any peer. Report to the orchestrator only.
