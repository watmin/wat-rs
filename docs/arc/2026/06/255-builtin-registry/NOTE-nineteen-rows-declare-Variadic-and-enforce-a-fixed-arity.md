# NOTE — nineteen registry rows declare `Variadic` and enforce a fixed arity

> Found while verifying Stone 1c-e's `str` registration. **Pre-existing, not introduced by that
> stone** — `str` is the nineteenth member of the class, not its cause.

## The measurement

```
(:wat::core::str 1 2 3)     --check → exit 0
                            run     → ArityMismatch ":wat::core::str: expected 1 arguments, got 3"
```

`#[wat_intrinsic]` derives `Arity` from the handler's **Rust signature shape**, per its own doc:
*"N such `&WatAST` params ⇒ `Exact(N)`. A single `&[WatAST]` leading param ⇒ **`Variadic`** (the
slice is passed through directly; **no arity check in the shim**)."*

A handler written as `fn f(args: &[WatAST], …)` that then checks `args.len() != N` internally
therefore registers as `Variadic` while behaving as `Exact(N)`. **Nineteen rows do this:**

```
apply 2 · not 1 · contains? 2 · get 2 · conforms? 2 · show 1 · str 1 · form::matches? 2
map 2 · mapv 2 · foldl 3 · stream->vec 2 · find-last-index 2 · filter 2   (+5 more)
```

## Why it matters

**`arity` is one of the RULING's seven — "what they take."** A row claiming `Variadic` for a verb
that accepts exactly one argument is the registry answering wrongly about its own item 4.

And it has a live consequence: the checker's registry-arity door only fires for `Arity::Exact`, so
a `Variadic` row **skips the arity gate entirely**. `(str 1 2 3)` type-checks clean and raises at
runtime — a **check-says-yes / runtime-says-no** divergence, the mirror of the class
`:wat::core::Tuple`'s own arm guards against in its comment: *"answering it with an empty tuple
here would be a check-says-no / runtime-says-yes divergence — the exact class step ①b's Room 3 was
found by."*

## ⛔ What this is NOT

It is not a doc-block error. The rows declare their `@arg`s correctly — `str` declares exactly one
— and `check_args` verifies the `@arg` names against the handler's params. **The doc is right and
the derived `Arity` disagrees with it.** Nothing today cross-checks the two.

## The shape of a fix, not taken here

Three candidates, none crawled:

- derive `Arity` from the declared `@arg`s (as `#[wat_special_form]`'s fold already does) rather
  than from the Rust signature — the two would then agree by construction;
- have the macro read the handler's own `args.len() != N` guard — fragile, a body scan;
- a gate that fails when a `Variadic` row's handler contains a fixed-arity check — the
  *"impose the check and read the screams"* move, which would name all nineteen at once.

★ The first is the only one that removes the divergence rather than detecting it, and it is the
same motion the special-form fold already makes. It wants its own stone and its own crawl:
nineteen rows change arity, and a row that is *genuinely* variadic must not be caught by it.
