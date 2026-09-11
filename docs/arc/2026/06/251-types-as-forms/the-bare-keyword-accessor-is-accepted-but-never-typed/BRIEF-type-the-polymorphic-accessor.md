# BRIEF — type the polymorphic accessor (arc 236.2's placeholder, premise spent)

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`.
Never use worktrees. Do not touch `~/work/holon/` (the frozen root).

## Tree state

HEAD `1b8a25cbb`, clean, a long UNPUSHED local chain. `cargo build --release` clean.

```
floor   5345 run   5336 passed   8 failed   22 skipped     clippy 0
```

**A red start is expected.** Those 8 are tracked elsewhere and are NOT yours.

## The defect — ONE block, TWO bugs, opposite directions

`src/check.rs` ~:5795–5840, the bare keyword-accessor path. `(:field receiver)` — there is no
function named `:field`; this is the "keyword as accessor" shorthand (arc 234.3c).

```rust
let acceptable = match receiver_ty {
    Some(TypeExpr::Path(p)) => match env.types().get(p) {
        Some(TypeDef::Aggregate(_))      => true,
        Some(TypeDef::Enum(e))           => e.variants.len() == 1,   // a tagged variant
        _ => false,
    },
    Some(TypeExpr::Parametric { head, .. }) if head == "wat::core::HashMap" => true,
    Some(TypeExpr::Parametric { head, .. }) => /* singleton Enum test */,
    …
};
if acceptable {
    // HARVEST (236.2): silent-by-intent — polymorphic accessor placeholder.
    let ty = fresh.fresh();              // <-- ACCEPTED, AND NOT TYPED
    return …;
}
// otherwise -> UnknownCallee
```

**Bug ① — a false ACCEPT.** When `acceptable`, the result is a FRESH VAR, which unifies with
anything. A declared return type is never checked.

**Bug ② — a false REFUSE.** The `Parametric` arms cover `HashMap` and the singleton **Enum**;
a parametric **Aggregate** is absent, so it falls through to `UnknownCallee`. Arc 296 A-2 RELAND-5
added the parametric-variant arm and left the parametric-record case out.

### Measured, this binary

```
                                            declared      result
RECORD monomorphic   (:x c)  -> String      a LIE         ACCEPTED   ⛔ bug ①
RECORD parametric    (:x c)  -> i64         CORRECT       REFUSED    ⛔ bug ②  "unknown callee: :x"
RECORD parametric    (:x c)  -> String      a lie         refused        (right answer, wrong reason)
VARIANT parametric   (:has d)-> String      a LIE         ACCEPTED   ⛔ bug ①
────  the named accessor and {:keys} are CORRECT on every row  ────
RECORD  Plain/x      -> String              a lie         refused    ✓
VARIANT D.Has/has    -> String              a lie         refused    ✓
VARIANT {:keys has}  -> String              a lie         refused    ✓
VARIANT D.Has/has    -> i64                 CORRECT       accepted   ✓
```

Four-line repros:

```wat
;; bug ① — a lie, accepted
(:wat::core::defrecord :u::Plain [x <- :wat::core::i64])
(:wat::core::defn :u::c [c <- :u::Plain] -> :wat::core::String (:x c))

;; bug ② — the truth, refused
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::a [c <- (:u::Cell :- [:wat::core::i64])] -> :wat::core::i64 (:x c))
```

## ★ The fix is already in hand at the site

`acceptable` is computed by **resolving the receiver and looking up its `TypeDef`** — the exact
lookup needed to find the field's declared type. The block finds the type, confirms the shape
carries named fields, then discards the answer and returns a fresh var. **Return the field's type
instead of `fresh.fresh()`**, instantiating `type_params -> args` when the receiver is `Parametric`.

`instantiate_field_types` (`src/check.rs:17827`, landed in the previous stone) already does exactly
that substitution for `{:keys}`. **Reuse it. Do not write a second one.**

⚠ An UNKNOWN field name on a known receiver must become a located error, not a fresh var. Today it
is silently accepted.

## ★★ Why the deferral is spent

Arc 236.2 named this a *"polymorphic accessor placeholder"* — a stand-in for typed accessors that
did not exist. Records have had `:T/field` for a long time; variants got `:Enum.Variant/field` in
the previous stone (`090525650`). **There is nothing left for it to stand in for.**
`HAERESIS EST ITERVM ROGARE` — the deferral was reasoned against a world that has changed.

## Acceptance — every row, including the controls

```
RECORD monomorphic  (:x c) -> i64      CORRECT   accept
RECORD monomorphic  (:x c) -> String   a lie     REFUSE   (currently accepts — bug ①)
RECORD parametric   (:x c) -> i64      CORRECT   accept   (currently refuses — bug ②)
RECORD parametric   (:x c) -> String   a lie     REFUSE
VARIANT parametric  (:has d) -> i64    CORRECT   accept
VARIANT parametric  (:has d) -> String a lie     REFUSE   (currently accepts — bug ①)
(:nosuchfield r) on a known receiver             REFUSE, located
HashMap receiver — unchanged                     (already passes — MUST STILL PASS)
every named-accessor and {:keys} row above       (already passes — MUST STILL PASS)
a runtime twin: the correct read RUNS and prints
```

Probes under `tests/types/`. Nothing covers this today, which is why it survived.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** A fix needs `assignable` or any variance change. It does not — this is a lookup that
already happens, returning its answer instead of discarding it. If you reach for variance, stop.

**STOP-2.** The HashMap arm's behaviour changes. A HashMap's value type is not a named field; it is
a different question and is out of scope.

**STOP-3.** The floor's failure count rises above 8, or any "already passes" control breaks.

**STOP-4.** You find a FOURTH read path for a named field (beyond bare keyword, `:T/field`,
`{:keys}`). Report it with a four-line repro; do not fix it — the census is the orchestrator's.

## What to run

`cargo build --release`, your probes, and `cargo nextest run --release -E 'binary_id(wat::types)'`.
**Do not run `scripts/floor.sh`, do not run clippy** — the orchestrator measures centrally, once.
Foreground everything. Do not commit. Do not push.

## Report

The acceptance table with real exit codes, pre and post. Every STOP finding. Anything uncertain.
