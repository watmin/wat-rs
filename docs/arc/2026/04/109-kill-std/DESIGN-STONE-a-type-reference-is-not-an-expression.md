# DESIGN — a `(Head :- [args])` form is a TYPE REFERENCE and must never be macro-expanded

> **Blocker 5.** Directly blocks identity **2c** (builder ruled F1: this first, then 2c whole), and
> blocks ②-iii independently. Root filed as 251.8's one-node-two-roles.

## The failure, and it is NOT what the earlier notes said

Every prior record of this — the seam, the ②-iii NOTE — described it as *"`defrecord` mints a macro
under the record's bare name, so the list is macro-expanded before the checker sees it."* True, but it
names the shadowing as the defect. **Macroexpanding the form says something sharper:**

```clojure
(:user::R :- [T])   →   (:wat::core::kwargs-construct :user::R :- [T])
```

The companion macro fires and emits a **constructor call**. *That* is what lands in the type slot, and
the resulting diagnostic blames the innocent party:

> `malformed :wat::core::fn form: invalid type keyword: malformed type expression "[…]":
> function-type bracket needs a `:->` arrow`

The `[…]` it complains about is the binder vector, now an argument to `kwargs-construct`.

★ **So the defect is not that the name is shadowed. It is that a TYPE REFERENCE is being evaluated as
an EXPRESSION.** The shadowing only decides *which* wrong thing happens; without a companion macro the
same form is simply unresolvable (measured: a user `typealias` head gives *"call head — not a builtin,
not a registered function"*). Both are the same root wearing different clothes.

## Measured surface

```
(:wat::core::Vector       :- [:i64])        builtin       ✅   Rust-registered, no macro
(:wat::cache::Lru         :- [:i64 :i64])   typealias     ✅   no companion minted
(:wat::spawn::ServiceEvent :- […])          defenum       ✅   no companion minted
(:wat::cache::Entry       :- [:i64 :i64])   defrecord     ⛔   companion macro fires
(:wat::spawn::Launched    :- […])           defstruct     ⛔   companion macro fires
```

## ⚠ The shadowing is DELIBERATE and must not be reversed

`wat/Record.wat:95-108` — arc 294 item 9a moved the positional constructor to the PRIME `:ns::T'` so
the bare name could be the ergonomic kwargs constructor:

> *CONSTRUCTION ERGONOMICS FLIP. The bare type name `~fqdn` is now the KWARGS macro (order-free
> `(:ns::T :field 1 …)`); the raw positional ctor moved to the PRIME `:ns::T'`.*

So "stop minting the companion" reverses a ruled decision and is not on the table.

## The options

- **H1 — the expander learns which SLOTS are types** and skips them.
- **H2 — each companion macro detects `:-` and declines**, returning a form the type parser accepts.
- **H3 — parse type slots BEFORE macro expansion.**
- **H4 — the expander skips any list whose element 1 is the `:-` keyword.** Shape-based, not
  position-based: `(Head :- [args])` has the marker at index 1, while a DECLARATION
  (`(defn :name :- [T] …)`) has it at index 2. The two shapes are distinguishable without knowing any
  head's grammar.

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **H1** expander knows type slots (position) | YES | **NO** | YES | — |
| **H2** each companion macro declines on `:-` | **NO** | **NO** | **NO** | — |
| **H3** parse types before expanding | **NO** | **NO** | YES | — |
| **H4** expander skips a list with `:-` at index 1 | YES | YES | YES | YES |

**H4** is the operator's own argument, applied one level out. `:-`'s whole justification is that the
param-spec sits in a **RESERVED position** — *values were never legal there, which is why nothing
needs to sniff it.* By exactly that reasoning, `(Head :- [args])` can never be a value expression, so
the expander can decline it on shape alone. One test, no per-head list, and it fixes **every**
macro-minted type at once — `defrecord`, `defstruct`, `holon::defrecord`, and any future companion —
rather than the three we happen to know about.

**H1 fails Simple** — the expander would need the grammar of every declaration head to know which
slots hold types. That is the `is_type_bracket_candidate` / declarator-head-list shape, and this arc
has watched such a list be wrong **twice** (the codemod's missing `typealias`; `defn` never probed).
`[[feedback_scope_the_check_from_the_rule_not_the_diff]]`

**H2 fails all three** — a constructor macro inspecting its arguments to decide whether it is a type
is a macro doing type-system work; the fix must be replicated into every minting head's emitted
companion; and it treats the symptom at each mint site rather than the rule. It also risks
re-expansion, since the honest thing for it to return is its own input.

**H3 fails Obvious and Simple** — reordering two whole passes to fix one shape.

## ⚠ The risk, and it is the whole risk

**Does any legitimate macro call carry `:-` as its FIRST argument?** H4 would silently stop expanding
it. The reserved-position doctrine says no such call should exist, but doctrine is not a census — and
this arc has been wrong about exactly that kind of claim more than once. **The floor is the instrument,
and a corpus census of `(<macro-head> :- …)` shapes is owed before the strike, not after.**

## Acceptance

1. ★ `(:user::R :- [T])` as an annotation CHECKS, where `R` is `defrecord`-minted — the row this
   stone exists for.
2. The same for `defstruct` and `holon::defrecord`.
3. ★ **The constructor still works.** `(:user::R :field v)` — the kwargs form arc 294 item 9a exists
   to provide — must be untouched. This is the negative control: a fix that disables the companion
   macro would pass row 1 and break the language.
4. `(:wat::cache::Entry :- [K V])` and the other stdlib `defrecord`/`defstruct` types check.
5. Floor 4854/4854, clippy 0.
6. Identity 2c's three blocked bindings (`state-ty`, `record-ty`, `handle-name`) can then convert.

⚠ Row 3 is the one that bites. Rows 1-2 measure that the form is no longer expanded; only row 3
measures that it is still expanded **where it should be**.
