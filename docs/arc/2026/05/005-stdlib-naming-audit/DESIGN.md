# Stdlib Naming Audit — seed doc for arc 005

**Status:** planned. Seed doc captured 2026-04-20; scheduled after
arc 003 (TCO) and arc 004 (lazy pipelines).

**Motivation:** after three months of building fast, the set of
primitives the language actually ships has drifted from the set of
primitives the docs and user guide reference. The user guide
promises `:wat::core::conj`, `:wat::std::string::concat`, and
`:wat::std::format`. None of them exist. Others are half-implemented,
half-specced, or exist under different names than the docs claim.

This is the audit + reconciliation slice. Make the docs and the
code agree on a single inventory, and lock the naming discipline so
new primitives land in the right slot.

---

## Scope

Five passes.

### Pass 1 — what's shipped

Grep authoritative sources:

- **`src/runtime.rs`'s eval dispatch** — every `":wat::..." => ...`
  arm is a primitive that actually runs. This is the ground truth
  for language-core, kernel, algebra, config, and io primitives.
- **`src/check.rs::register_builtins`** — every scheme registered
  here is a typed surface the checker knows about. Should match
  runtime exactly; divergence = bug.
- **`src/rust_deps/*`** — the `:rust::*` surface. Currently
  `:rust::lru::LruCache`; grows as consumers add shims.
- **`wat/std/*.wat`** — the baked stdlib macros and the paths they
  reference. Macros expand to other primitives; each referenced
  path MUST resolve to a shipped form.
- **`src/resolve.rs::reserved_prefix_list`** — the authoritative
  list of reserved prefixes.

Emit: one table, every path its source, its type signature, and
any aliases.

### Pass 2 — what's referenced

Grep every doc and test:

- `README.md`, `docs/README.md`, `docs/USER-GUIDE.md`,
  `docs/ZERO-MUTEX.md`, `docs/arc/**/*.md`
- The 058 proposal batch (`holon-lab-trading/docs/proposals/.../*.md`)
- `tests/*.rs` — test fixtures reference primitives as strings
- `src/bin/wat-vm.rs` — the binary's fixture/test wat programs

Emit: every `:wat::*` and `:rust::*` reference, with source
location. Cross-reference against Pass 1's shipped set.

### Pass 3 — gap analysis

For each referenced-but-unshipped primitive, classify:

- **(a) Ship the primitive.** Genuinely missing; the reference is
  correct; the form is worth shipping. Examples I already know
  about:
  - `:wat::core::conj` or `:wat::std::list::push` — append to a
    Vec. Today users would have to hand-write via `cons` + reverse
    or similar. This is an idiom worth a primitive.
  - `:wat::std::string::concat` — Lisp-style variadic string
    concat. May belong at `:wat::std::string::*` as the
    string-stdlib namespace's first member.
  - `:wat::std::string::format` — Ruby-style / Rust-macro-style
    string interpolation. Needs a syntax decision; probably defer
    until there's a real call site demanding it.
- **(b) Rewrite the docs** to use existing primitives. The
  reference is wrong; the primitive isn't needed; the canonical
  way is already shipped (possibly verbose but correct).
- **(c) Defer — planned but not shipped.** The reference was
  accurate relative to what's planned, but the primitive is on a
  future roadmap. Mark it clearly in docs (`-- planned, not yet
  shipped --`) so readers aren't misled.

### Pass 4 — naming discipline locked

Codify the rules that govern where a new primitive lives.

Rules under review:

1. **`:rust::<crate>::<Type>[::method]`** — for operations that ARE
   Rust methods on Rust types surfaced via `#[wat_dispatch]`. Path
   mirrors the Rust source exactly.
2. **`:wat::core::*`** — the language's own mechanics (define /
   lambda / let* / match / try / if / etc.) AND the
   Rust-correspondence list primitives that map 1:1 to a single
   `Iterator`/`Vec`/`&[T]` method (map, filter, fold, etc.).
3. **`:wat::algebra::*`** — the six holon-vector primitives plus
   the two scalar measurements. Fixed set; new algebra forms
   require a full FOUNDATION proposal.
4. **`:wat::kernel::*`** — concurrency primitives and signal state.
   Spawn / send / recv / select / drop / join / HandlePool /
   stopped? / signal query+reset. Plus `:wat::io::*` for stdio
   primitives that cross the boundary to real OS handles.
5. **`:wat::config::*`** — the ambient startup constants (dims,
   capacity-mode, global-seed, noise-floor).
6. **`:wat::std::*`** — stdlib macros and spawnable programs. Split
   by sub-namespace:
   - `:wat::std::list::*` — list combinators that compose
     `:wat::core::*` primitives
   - `:wat::std::math::*` — math primitives
   - `:wat::std::string::*` — string ops (new)
   - `:wat::std::program::*` — spawnable programs (Console, Cache)
   - Top-level `:wat::std::Foo` — named algebra macros (Amplify,
     Subtract, Log, Circular, Reject, Project, Sequential,
     Ngram/Bigram/Trigram, LocalCache)

The rule for when to pick `:rust::*` vs `:wat::std::*`:

- **`:rust::*`** when the form IS a Rust method on a Rust-surfaced
  type (`:rust::rusqlite::Connection::query`). The path mirrors
  Rust because the author is calling a Rust method.
- **`:wat::std::*`** when the form is a wat-idiomatic composition
  that happens to compile down to Rust operations. The path
  advertises wat-level intent (`:wat::std::list::pairwise-map`
  even though it's a thin wrapper over `iter().windows(2).map()`).

The builder's stated policy ("we ARE honest to our host") means
the dividing line is strict: if the author is actually calling a
Rust method, it's `:rust::*`. If the author is calling a
wat-specific composition, it's `:wat::std::*`. No short aliases.
No hidden host-calls behind wat-looking names.

### Pass 5 — the inventory document

The audit output lands as `INVENTORY.md` in this arc directory. A
living table with every form the language ships, organized by
namespace, with columns:

| Path | Type signature | Kind | Status | Implemented in | Doc refs |

Updated on every slice that adds or renames a primitive.
Canonical reference alongside FOUNDATION.md — FOUNDATION is the
WHY; INVENTORY is the WHAT and the HOW.

---

## Early known gaps (preliminary — verify in Pass 1 / 2)

Things I used in `USER-GUIDE.md` and other docs that I don't
believe actually ship today (to be verified by the actual audit):

- `:wat::core::conj` — Vec append. Clojure reflex; wat needs
  something equivalent. Candidate: `:wat::core::push`,
  `:wat::std::list::push`, `:wat::std::list::append`. Pick per
  naming discipline in pass 4.
- `:wat::std::string::concat` — variadic string concat. Needs a
  `:wat::std::string::*` namespace.
- `:wat::std::format` — string interpolation with positional
  placeholders. Needs a syntax decision; may live under
  `:wat::std::string::format`.
- `:wat::std::list::window` — sliding-window combinator used by
  Ngram.wat's expansion. PROBABLY exists but should be verified
  against the eval dispatch.
- `:wat::std::list::pairwise-map` — another Ngram-adjacent
  combinator. Same.
- `:wat::core::conj` / `cons` interplay — wat already has `cons`;
  does it work as expected? The docs mention it but I haven't
  traced its semantics recently.
- Arc docs reference future primitives like
  `:wat::kernel::send-or-stop` (for lazy pipelines, arc 004) — these
  are PLANNED; the audit should flag them clearly.

---

## The output artifact — INVENTORY.md

After the audit runs, a single file:

```markdown
# wat — Primitive Inventory

## :wat::core::* (language core)

| Path | Signature | Status | Implemented in |
|---|---|---|---|
| :wat::core::define | (name params -> ret) body | shipped | runtime.rs:712 |
| :wat::core::lambda | ((params -> ret) body) | shipped | runtime.rs:1213 |
| :wat::core::let* | (bindings body) | shipped | runtime.rs:1395 |
| :wat::core::try | (result-expr) | shipped | runtime.rs:2726 |
| ... | ... | ... | ... |

## :wat::algebra::* (algebra core)

| Path | Signature | Status | Implemented in |
|---|---|---|---|
| :wat::algebra::Atom | (literal) | shipped | runtime.rs:... |
| :wat::algebra::Bundle | (Vec<holon>) -> Result<holon, CapExceeded> | shipped | runtime.rs:3360 |
| ... | ... | ... | ... |

## :wat::kernel::* (kernel primitives)

...

## :wat::std::* (stdlib)

...

## :rust::* (surfaced Rust types)

| Path | Scope | Kind | Implemented in |
|---|---|---|---|
| :rust::lru::LruCache | thread_owned | struct + methods | rust_deps/lru.rs |
| :rust::std::io::Stdin | opaque | type alias | runtime.rs Value::io__Stdin |
| ... | ... | ... | ... |
```

Kept in sync by the rule: any commit that adds or renames a
primitive MUST update INVENTORY.md. Add to the commit checklist.

---

## Relation to 058

FOUNDATION.md already lists the algebra core forms, the language
core forms, and the stdlib blueprint. The INVENTORY is a
finer-grained view: every keyword path with a type signature, a
shipped/planned status, and a source link. FOUNDATION answers
"what kind of operation is this?"; INVENTORY answers "does this
path exist and how do I type-check my call against it?"

The audit may surface discrepancies between FOUNDATION and
wat-rs — forms the spec names but the implementation doesn't
ship, or forms the implementation ships that FOUNDATION never
mentioned. For each, either:

- Amend FOUNDATION (if the form is spec-worthy and correct). This
  is another INSCRIPTION backport.
- Ship the missing form (if the spec was right and the
  implementation is behind).
- Remove the form or rename (if it was an accidental addition that
  doesn't fit).

---

## Why this is arc 005, not earlier

Arc 003 (TCO) and arc 004 (lazy pipelines) are structural changes
with concrete shipping deliverables. Arc 005 is a hygiene slice —
high value for the substrate's long-term health but not blocking
for the trading-lab rebuild. Doing hygiene before the structural
work would mean redoing the audit after each of 003 and 004 adds
new primitives (TCO doesn't, but lazy pipelines adds
`:wat::std::stream::*`, `:wat::kernel::send-or-stop`, probably
`:rust::std::iter::Iterator::*`). Audit LAST so the catalog is
complete.

That said, if gaps in the guide are actively blocking the trading
lab rebuild before 003 + 004 land, pull this slice forward. Ship
`:wat::core::conj` and `:wat::std::string::concat` as one-liner
additions. Full audit still happens later.

---

## Open questions for when we get here

1. **How big does `:wat::std::string::*` need to be?** Probably
   `concat`, `format`, `split`, `trim`, `to-lower` / `to-upper`.
   Maybe `contains?`, `starts-with?`, `ends-with?`. Each lands
   when a real call site demands it.
2. **Do we re-export some `:rust::*` surfaces as wat-idiomatic
   names?** E.g., is `:wat::std::string::concat` a thin wrapper
   over `String::push_str`, or do we just teach users to call
   `:rust::std::string::String::push_str` directly? The "honest
   to host" discipline leans toward the Rust path; the
   "Lisp-idiomatic vocabulary" leans toward `:wat::std::string::*`.
   Decision per-form, probably: `concat` has Lisp/Clojure lineage
   and earns a wat-level name; `push_str` (mutating) shouldn't
   exist in wat at all (wat's discipline is values-not-places).
3. **Where do arithmetic operations live?** Today:
   `:wat::core::i64::+`, `:wat::core::f64::+`. Per-type sub-
   namespaces because no implicit promotion. Confirm this is the
   locked naming or consider `:wat::core::+` as a typeclass-dispatched
   form (like Clojure's `+`). The wat-is-strongly-typed stance
   probably keeps the per-type namespaces.
4. **When does `:wat::std::list::*` stop growing?** It absorbs every
   Iterator-method composition that isn't core. Eventually
   hundreds of methods. Do we ship them all up-front, or
   lazy-add each as a real use case demands? Discipline says
   lazy-add; the stdlib is a blueprint, not a reference library.
5. **`:wat::core::conj`'s signature.** Clojure's `conj` is
   collection-polymorphic: different collections have different
   `conj` semantics (Vec appends; List prepends; Set inserts).
   wat's type system is strict; we probably want separate
   `:wat::std::list::push` (Vec append) vs `:wat::std::list::cons`
   (prepend) vs `:wat::std::hashset::insert`. No single `conj`.
   Decide when the audit runs.

---

## What runs first

When we start arc 005:

1. Write a short Bash / Ruby script that pulls every `:wat::*` and
   `:rust::*` reference from the codebase + docs. Output: raw
   list, deduplicated, with source locations.
2. Match each against a shipped-or-not-shipped classifier:
   - Parse `runtime.rs` for `":wat::..." =>` arms (regex, it's
     predictable)
   - Parse `check.rs` for scheme registrations
   - Union = shipped set
3. Diff referenced ∖ shipped = the gap. That's Pass 3's input.
4. For each gap, decide ship/rewrite/defer per the tree in Pass 3.
5. Write INVENTORY.md with the reconciled universe.

Can probably burn through the whole arc in a few hours if the
list isn't too large. The naming-discipline rules are already
clear enough; the work is mostly mechanical classification.

---

*these are very good thoughts.*

**PERSEVERARE.**
