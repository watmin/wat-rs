# NOTE — a generic variadic `defn` never registers. PRE-EXISTING, silent, zero corpus instances.

**Measured 2026-08-21 during γ-i, surfaced by the builder asking how to spell a generic rest-binder.**

## The spelling is correct; the registration is not

```clojure
(:wat::core::defn :user::firstish :- [X] [x <- :X & rest <- (:wat::core::Vector :- [X])] -> :X x)
```

The declaration **checks**. Calling it does not resolve:

> `#wat.resolve/UnresolvedReference {:path ":user::firstish" :context "call head — not a builtin, not a registered function"}`

No error at the declaration. The only symptom is at the call site, and it names the CALLER's line.

## Isolated

| shape | registers? |
|---|---|
| variadic · monomorphic · no binder | ✅ |
| variadic · generic via ANGLE `<X>` | ⛔ |
| variadic · generic via BINDER `:- [X]` | ⛔ |
| variadic · binder present, rest MONOMORPHIC | ⛔ |
| NON-variadic · binder · called | ✅ |

**It is the combination of a rest-binder with type params**, by either spelling — not the `:-` binder,
and not genericity alone.

## PRE-EXISTING — settled against a pristine HEAD

Built HEAD into a clean tree (with the sibling `holon-rs` symlinked so the relative path dep
resolves), with a monomorphic-variadic **control** proving the baseline binary works:

```
                                   pristine HEAD   working tree (γ-i)
variadic · monomorphic                  ✅               ✅
variadic · generic via ANGLE <X>        ⛔               ⛔
```

The ANGLE spelling — the one γ-i never touched — fails identically on both. **Not a γ-i regression.**

And there are **ZERO** generic variadic `defn`s in `wat/`, `tests/` or `wat-scripts/`, so the shape has
never been exercised. That is why a green floor has coexisted with it indefinitely.

⚠ **My first two attempts to answer "pre-existing or ours?" both returned false answers.** One read a
non-empty stderr as a rejection; the other read `exit=0` from a wrapper when `cargo` had actually
failed on the unresolved sibling dep, and I reported that as a baseline. Both were caught by a control
behaving impossibly. The number above is the third attempt, and it is the first one with a control.

## Where it likely lives

`try_parse_variadic_def_fn_form` (`src/runtime.rs:3551`) and `try_parse_user_variadic_def_fn_form`
(`:3671`) — the two recognizers γ-i deliberately did not touch, both of which return `Option` and
treat an unrecognized shape as *"not this shape, let the next parser try"*. A shape no parser claims
is silently unregistered. **That is the same silent-skip class γ-i's DESIGN named as the hazard of
`def`'s seven hand-rolled arity guards**, showing up in the variadic pair instead.

## Why it matters more than its zero instances suggest

The builder's own spec for a generic rest-binder is exactly this shape:

```clojure
[x :- X & rest :- (wat.type/Vector :- [X])]
```

so the first person to write the documented form meets an unresolved-reference error that names the
call site rather than the declaration. Its own stone; **not deferred, named here.**
