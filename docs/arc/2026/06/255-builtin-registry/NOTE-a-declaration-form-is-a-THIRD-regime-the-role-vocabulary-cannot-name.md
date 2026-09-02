# NOTE — a declaration form is a THIRD regime, and `SpecialFormRole` cannot name it

> Measured while drawing Phase 1a-β. **The stone I was about to draw cannot be built as drawn**, and
> the reason is a real gap, not a scoping problem.

## What I went to measure

`[[NOTE-the-sloppy-registries-a-measured-census]]` found five hand-lists answering *"what kind of
form is this head?"*, mutually inconsistent. The annihilation is a registry query on `@Category`
(`Declaration` exists; `Category` is generated from `wat/runtime-meta.wat`). The RULING's order is
forced: **registry answers → consumer asks → duplicate dies.** So 1a-β = register the population.

I picked `freeze::is_declaration_form` as the first target on one measured property: **it is the only
one of the five with no `starts_with`** — a pure set, exactly expressible as a registry query. Nine
names:

```
def · defmacro · defstruct · structtype · defenum · newtype · typealias · defalias · defsurface
```

## ⛔ THE BLOCKER — the registration gate demands a regime these forms do not have

`src/intrinsic/mod.rs` · `every_special_form_carries_check_and_eval_impls` requires every
`Kind::SpecialForm` row to name a `Check` impl **and** an `Eval` impl. `SpecialFormRole`'s own doc
says what those mean:

```rust
pub(crate) enum SpecialFormRole {
    /// Static type inference — `src/check.rs`'s `infer_*` fns.
    Check,
    /// Per-invocation evaluation — `src/runtime.rs`'s eval match.
    Eval,
    /// Per-invocation evaluation in tail position (TCO).
    Tail,
}
```

**Both are per-invocation-or-inference regimes. A declaration form is processed at FREEZE time** —
`src/freeze.rs`'s `is_declaration_form` gate and `src/declare/register.rs`'s `register_defines` /
`register_defclause` — before evaluation exists and outside `infer_*`.

Measured, name by name (count of the FQDN string per file; a count is a floor on annotatability, not
a proof that an annotatable fn exists):

| name | `check.rs` | `runtime.rs` | `freeze` | `declare/` | `types.rs` |
|---|---:|---:|---:|---:|---:|
| `def` | 8 | 5 | 3 | 7 | 2 |
| `defmacro` | 1 | 1 | 2 | 0 | 0 |
| `defstruct` | 1 | 1 | 3 | 1 | 1 |
| `defenum` | 1 | 1 | 2 | 1 | 2 |
| `newtype` | 1 | 1 | 2 | 0 | 1 |
| `typealias` | 1 | 1 | 2 | 0 | 1 |
| **`structtype`** | **0** | 1 | 2 | 1 | 1 |
| **`defalias`** | 1 | **0** | 1 | 1 | 0 |
| **`defsurface`** | **0** | **0** | 2 | 0 | 6 |

★★★ **`defsurface` has neither.** It is handled entirely at freeze + type-registry time. `structtype`
has no checker arm; `defalias` has no runtime arm.

**This is not a defect in those three forms. It is the gate encoding an assumption — *every special
form is checked and evaluated* — that a freeze-time declaration form does not satisfy.** The
assumption held for every form registered so far (`if`/`let`/`fn`/`match`/`and`/`or`) because all six
are expression forms. 1a-β is the first stone to meet the other kind.

## ⚠ Two things I checked and was WRONG about — recorded so neither is re-derived

- **`structtype` is not a retired name.** Three hand-lists carry it and I suspected a dead entry
  (Stone 241.8's "defstruct replaces struct — HARD CUT"). It is live: `types.rs:3999` —
  *"structtype is the low-level primitive `defstruct` (now a macro) expands to."* My probe returned
  `UnresolvedReferences` because a bare `structtype` does not mint the constructor/accessors the
  `defstruct` macro layer adds — a defect in the probe, not the form.
- **`defalias` is not retired either.** It appears in `RETIREMENT_TABLE` as a **replacement**
  (`:wat::runtime::define-alias` → `:wat::core::defalias`), which is the opposite of retired.

## The measured population, for whoever draws this next

The five hand-lists' union is **20 entries — 19 names + one prefix (`:wat::config::set-`) — and
exactly ONE (`:wat::core::let`) is registered today.**

★ **No name appears in all five.** `def` is in four (absent from `is_mutation_head` — the live drift
the census found). The `defmacro`/`defstruct`/`structtype`/`defenum`/`newtype`/`typealias` block sits
in exactly the mutation/declaration trio. `derive`/`extend-type`/`defclause` sit only in the
`declare/` pair. **Five lists, five different populations, no two the same.**

## ⬜ What this NOTE does NOT decide

It does not pick the fix. Naming a third regime, exempting declaration forms from the gate, or
finding another vehicle for `@Category` are different rulings with different blast radii, and this
arc's standing rule is that a fork like this is argued against the four questions in the main chat
rather than improvised at the end of a measurement.

★ What it does establish: **the blocker is real, it is one gate and one enum, and it was invisible
until a stone tried to register a form that is neither checked nor evaluated.**
`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`
