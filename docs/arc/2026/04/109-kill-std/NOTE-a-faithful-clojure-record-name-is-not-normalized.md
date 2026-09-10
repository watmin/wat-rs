# Faithful-Clojure translates in every position but one — the record NAME — so no `defrecord` the converter emits can be read back

**Filed 2026-09-10, arc 278 (`strike-no-rule-that-cannot-compile`), grounded here in arc 109
because this is the faithful/colon-mode surface. Measured, not fixed.**

## What was actually measured

The two `to-faithful-clojure-*` codemods ran for the first time ever at `2a9de244a` and both emit,
byte-identically, from `wat/source.wat`:

```clojure
(wat.core/defrecord wat.source/File
  [path   :- wat.type/String
   source :- wat.type/String])
```

Both codemods' own headers claim *"Gate = round-trip parse of the output."* **That output does not
parse.** Driven at `2a9de244a` on `./target/release/wat`:

| # | form | verdict |
|---|---|---|
| **A** | `(wat.core/defrecord wat.source/File [path :- wat.type/String])` | ⛔ `#wat.macro/ProgramBodyEvalFailed` |
| **B** | `(wat.core/defrecord :wat::source::File [path :- wat.type/String])` | `DuplicateType :wat::source::File` — i.e. it **got through**, and only collided with the stdlib's own declaration |
| **C** | `(:wat::core::defrecord wat.source/File [path <- :wat::core::String])` | ⛔ identical failure to A |
| **B′** | `(wat.core/defrecord :zz::Fresh [path :- wat.type/String  n :- wat.type/i64])` | ✅ prints `"ok"` |
| **A′** | `(wat.core/defrecord zz.rec/Fresh [path :- wat.type/String  n :- wat.type/i64])` | ⛔ identical failure to A |

⭐ **B′ is the load-bearing row.** Faithful head, faithful `:- ` arrow, faithful `wat.type/…` in
*type* position — all of it translates and the program runs. **Swap only the record NAME to
faithful form (A′) and it dies.** A and C failing identically shows the head spelling is
irrelevant: this is the name argument alone.

## The site

`wat/Record.wat:171` — `fqdn-str (:wat::core::keyword/to-string fqdn)` — raising
`:wat::core::keyword/to-string: expected keyword, got wat::WatAST`. The macro receives the record
name as an un-normalized AST symbol and asks it for a keyword's string.

## ⛔ The mechanism is NOT established here, and the two candidates have different cures

The normalize pass that rewrites `a.b/c` → `:a::b::c` is arc 251 stone 251.1b
(`src/resolve/mod.rs:314`), and its own test is named
**`namespaced_symbol_head_normalizes_to_keyword`** (`src/resolve/mod.rs:317`) — *head*. Neighbouring
tests establish that it deliberately skips quoted forms, `match` patterns, and quasiquote templates.

That leaves two readings, and this NOTE does **not** pick between them:

1. **Ordering** — macro expansion runs before normalize, so `defrecord` always sees the raw symbol.
   Cure: normalize before macro expansion, or normalize macro arguments.
2. **Position** — normalize runs first but does not rewrite a *definition-site* name (as opposed to
   a reference). Cure: teach it the definition-site positions.

⚠ Reading 2 is complicated by B′: `wat.type/String` in *type* position DOES end up correct. That
may be normalize doing its job, or `defrecord`'s own expansion handling the `:- ` field syntax
itself. **Distinguishing these is the first work, and it is not done.** Anyone who fixes this
without first settling which reading is true is guessing at a cure.

## Why it matters

- Both `to-faithful-clojure-net.wat` and `to-faithful-clojure-rete.wat` are **recorded migrations**
  — `CLAUDE.md` names `wat-scripts/fixes/*.wat` as the shape to copy. Their stated round-trip gate
  does not hold, and until `2a9de244a` neither had ever run, so nobody could discover that.
- The failure is a macro-internal type error (`expected keyword, got wat::WatAST`), not a located
  syntax diagnostic. A user writing faithful-Clojure by hand gets a message about
  `keyword/to-string` and no indication that the *record name* is the thing to change.

## Scope

Orthogonal to whether rete rules compile, which was `strike-no-rule-that-cannot-compile`'s subject —
that strike's gate is green and this is out of its enumerated blast radius. Filed, not fixed.
