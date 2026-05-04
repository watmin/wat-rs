# Arc 152 — Local-bindings reflection (the pry-like introspection finisher)

**Status:** SCRATCH stub captured 2026-05-03 evening, post-arc-144
closure (the "nothing is special" trio of arcs 148/146/144 just
shipped). Designed for future-self when arc 109 v1 closes and the
next-leg work begins.

## The user's framing — captured

> *"how can we query 'what local bindings do i have?' - that feels
> like the last thing?... with what we have now we can get some
> struct and then query 'what funcs does this thing have?' ...
> we've basically got everything we need to build something like
> a ruby pry for an introspection perspective?"*

The principle: arc 144's uniform reflection foundation makes every
**registered** symbol queryable (UserFunction / Macro / Primitive /
SpecialForm / Type / Dispatch). What's left is the **scope-bound**
symbols — the local bindings of the current call frame. Once that
ships, the substrate has every primitive a pry-like REPL needs.

## The pry mapping — what we have vs what we need

| Pry feature | wat substrate equivalent | Status |
|---|---|---|
| `show-source method` | `:wat::runtime::body-of` | ✅ shipped (arc 143/144) |
| `show-doc method` | `:wat::runtime::doc-string-of` (future) | ⏳ paved road; pending arc 141 |
| `cd object` (drill into type) | `:wat::runtime::lookup-define :wat::core::Type` | ✅ shipped (arc 144 Binding::Type) |
| `ls type` (members of a type) | derive from `lookup-define` of the type + walk struct field accessors | ⏳ likely works post-arc-144; verify in arc 152 audit |
| `whereami` (current location) | `:wat::core::location` (errors carry coordinates per arc 138) | ✅ partial (errors carry; need explicit primitive) |
| **`ls` (local bindings in scope)** | **`:wat::runtime::local-bindings`** | ❌ **MISSING — this arc** |
| `wtf?` (backtrace) | arc 113 cascading runtime errors carry the backtrace | ✅ partial (errors carry) |

**Arc 152 closes the `ls` gap.** Once this ships, every pry primitive
has a substrate equivalent.

## What this arc would ship — sketched

### A new substrate primitive: `:wat::runtime::local-bindings`

Returns a `:Vec<:wat::holon::HolonAST>` describing every local
binding currently in scope. Each binding carries `name`, `type`,
and (optionally) `value`.

```scheme
(:wat::core::define
  (:user::demo (acc :i64) (xs :Vec<i64>) -> :i64)
  (:wat::core::let*
    (((doubled :Vec<i64>) (:wat::core::map xs :user::double)))
    ;; At this point, local-bindings would return:
    ;;   [(name="acc"     type=:i64       value=10)
    ;;    (name="xs"      type=:Vec<i64>  value=[1,2,3])
    ;;    (name="doubled" type=:Vec<i64>  value=[2,4,6])]
    (:wat::runtime::local-bindings)))
```

### Open architectural questions for the future-self drafting this arc

**Q1 — Scope tier:** current frame only? current + enclosing? full
call stack? Pry's `ls` shows current scope by default; `--all` adds
ancestors. Suggest defaulting to **current frame only** for the
substrate primitive; future-arc REPL consumer can compose scope-
walking on top.

**Q2 — Return shape:** `Vec<HolonAST>` per binding (consistent with
the rest of the reflection trio's output shape) vs flat
`Vec<(String, TypeExpr, Value)>` tuple. HolonAST is the consistent
shape but adds verbosity; tuples are easier to consume from wat
code. Probably HolonAST per the reflection trio's precedent.

**Q3 — Value capture:** include the bound value? Or just name +
type? Pry shows values; that's the debugging affordance. Including
values requires the primitive to access the runtime env at call-
time; that's already how built-in primitives work (they receive
evaluated args). Question is whether the env walker can produce
a Value for each binding without re-evaluation.

**Q4 — Where can it be called?** Only inside a function/let* body
(real locals exist)? Available at top-level too (returns empty
Vec)? In a deftest? Suggest: **legal everywhere; returns empty
Vec at top-level / outside any binding scope.**

**Q5 — What about parameters vs let* bindings vs match patterns?**
Function params are bindings. let* binders are bindings. Match-arm
binders are bindings. Lambda params are bindings. All should
surface uniformly; the `kind` field could distinguish if useful
(e.g., `:param` / `:let-binding` / `:match-binder`).

**Q6 — How does this work substrate-side?** The runtime env is
already a stack of scopes (Environment in `src/runtime.rs`). The
primitive walks the current scope's bindings + emits HolonAST
descriptions. The Env walk is read-only; no Mutex; Tier 1 + Tier 2
mechanics already in place.

## Why arc 152 is the *last* introspection primitive

Post-arc-152, the substrate has:
1. **Registered-symbol query** (arc 144) — every kind of named
   substrate entity reflectable
2. **Local-binding query** (this arc) — every scope-bound binding
   reflectable
3. **Source query** (arc 143/144 body-of) — the body of any callable
4. **Docstring query** (arc 141 future) — the docs of any callable
5. **Location query** (arc 138 partial) — coordinates carry through
   errors; explicit `current-location` primitive would close this

The pry-like REPL becomes a wat-level program composing these
primitives. No further substrate primitives needed for
introspection per se.

## Cross-references

- arc 144 INSCRIPTION — uniform reflection foundation
- arc 143 INSCRIPTION — `:wat::runtime::*` namespace + reflection trio
- arc 141 DESIGN — docstrings (paved road via Binding.doc_string)
- arc 138 INSCRIPTION — errors carry coordinates
- arc 113 INSCRIPTION — cascading runtime error messages

## Status

Stub captured. Future arc when the next-leg work names a REPL or
debugging consumer that needs `ls` to do its job. Until then, the
substrate's introspection story is "everything but local
bindings" — which is enough for arc 144's static-reflection
purpose, but not enough for pry-shape interactive debugging.

The substrate's introspection completeness is one substrate
primitive away. **This is the last thing.**
