# DESIGN — a fence-local binder may not shadow a rete variable

## Why

`check_fence_interior` (`src/rete/validate/typing.rs:516`) walks a `where` fence's interior and
type-checks every comparator it finds — **except inside a `let` body or a `match` arm**, which it
returns from without descending. Driven at HEAD: an ill-typed predicate there is **accepted**.

```wat
(:wat::rete::where (:wat::rete::core::let [x 1] (:wat::rete::core::string::= ?k 1)))   ;; ACCEPTED
```

The bound is one day old, drawn by the `accumulate :from` strike under STOP-2, and its own doc says
it chose *"narrowing what is walked, not guessing a shadowing rule"* — it **deferred** the question.
Its stated justification is that a fence-local `let` can **shadow** a rule-wide `?var` at a
different type, so a walk with no scope stack would read a shadowed name at the wrong type and
refuse legal, firing code. ⭐ **That hazard is real** — driven: `(let [?k "str"] …)` compiles and
runs beside a condition binding `?k` to `i64`.

## ⭐ But the bound buys tolerance for something NOTHING USES

Measured at `2f8c221e6`, with the grep **anchored on a known positive** first:

| | count |
|---|---|
| `.wat` sites binding a `?`-prefixed name in a `let`/`match` | **0** |
| fences using a `let` at all | **1** |

The single site is `where-inline-computed.wat:166`:

```wat
(:wat::rete::where (:wat::rete::core::let [x ?k] (:wat::rete::core::i64::> x 100)))
```

Binder `x` — a plain local holding a rete variable's **value**. The legitimate use, and untouched
by this change.

## ⛔ THE ONE CONTRACT DECISION

> **A `?`-prefixed name may not be a BINDER in a fence-local `let` or `match` pattern. It is
> refused at rule-compile time, by its own error variant.**

This is the constraint-engineering rung, not the check rung: `?k` **means** "the rete binding for
k". A local silently redefining it inside a fence is a form with no legitimate use, and making it
**unrepresentable** is what retires the scope-stack requirement — the walk cannot read a shadowed
name at the wrong type if no name can be shadowed.

⚠ **Rejected alternative:** build the scope stack and keep shadowing legal. It is strictly more
machinery to preserve an ability with zero users and no defensible reading.

⚠ **Scope is the FENCE ONLY.** Outside rete, `?x` is an ordinary symbol with no binding meaning;
outlawing it there would be an unrelated language change with its own corpus question.

## ⛔ What this does NOT do, and the honest reason

**It does not make `let`/`match` bodies type-checked.** Look again at the one real site: to check
`(i64::> x 100)` the walker must know `x` is an `i64` — and `x` is not in the rule-wide bind map,
not a field, and not a literal, so `resolve_operand_type` answers *not knowable* and the operand is
skipped. **Actually checking those bodies needs a local type environment threaded through the
walk**, which is separate, larger work.

So this stone **retires the justification** for the bound and removes the hazard. Whether to then
descend into the body is a follow-on that can be decided on its own merits, against a substrate
where shadowing is impossible.

## Files

| file | change |
|---|---|
| `src/rete/validate/error.rs` | the new variant + `Display` |
| `src/rete/validate/typing.rs:516-533` | refuse a `?` binder in the `let` binding vector and in each `match` pattern |
| `tests/rete/` | probe: refusal fixtures + the positive control |

## Out of scope = REJECTED

- **Descending into `let`/`match` bodies.** Needs the local type env; see above.
- **`?`-prefixed binders outside a rete fence.** Different question, different corpus.
- **The scope stack.** Rejected above, affirmatively.
