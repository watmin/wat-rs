# NOTE — the flip rewrites EVERY identifier, so an identifier must become a TYPE before 300.2 — not a `String` re-spelled twice

**Filed 2026-08-01, out of arc 278's `Token.bindings` work, at the builder's direction. Not fixed.
Tracked HERE rather than in 109 because 109 filed the storage GAP and named its forcing function —
*"the cheap moment to decide is before the flip"* — and this arc IS the flip.**

> **The builder, this session:** *"we will be abandoning colon-quoted-symbols in the very near
> future… `:wat::core::+` is going to be `wat.core/+`… the colon-quoted thing was a bad idea we
> haven't circled back to kill."* And: *"we both know — exhaustively — this is not a perf thing,
> it's a correctness thing."*

## The finding

An identifier is a heap `String` at both layers:

```rust
// src/value/value.rs
wat__core__keyword(Arc<String>),

// crates/wat-reader/src/identifier.rs:79
pub struct Identifier { name: String, scopes: BTreeSet<ScopeId> }
```

**300.2–300.4 rewrite every identifier in the corpus** (37 stdlib files / ~5857 `::`; 1173 `.wat`;
~192 inline-wat Rust sites), and **300.5 changes what the reader accepts**. Both are identifier
work. With `String` identifiers, the re-spelling is a text migration across the five surfaces arc
278's 24t measured the hard way — *"a rename touches FIVE surfaces and the codemod reaches one and a
half"* — including the one that caused its 2530-error cascade: **`.wat` string literals that BUILD
or PARSE keywords**, which a form-tree codemod structurally cannot reach. With an identifier TYPE,
the spelling lives behind that type's parse/print and the drives are a source codemod plus one door.

Doing storage after the drives converts every identifier **twice**. That is this arc's own law —
*"convert, THEN retire"*, because *"two accepted surfaces is still two readers"* — applied one layer
down, in the value model.

## What a wat identifier actually carries (grounded this session, not recalled)

- `Identifier { name, scopes }` — **two fields**; `grep -c meta` in `identifier.rs` → **0**. There is
  no open metadata map.
- `WatAST::Symbol(Identifier, Span)`, doc: *"Bare identifier, as in `x`, `role`, `tmp`… **the only
  places the language admits bare names**."*
- `WatAST::Keyword(String, Span)`, doc: *"Keywords carry no scope tracking — their full-path
  spelling already disambiguates `:my::app::foo` from `:my::macro::foo`."*
- `eval_quote` (`runtime.rs:10089`) returns `Value::wat__WatAST`, *"Quote is how programs become
  holons without running"*; the reverse unwrap is `:10470`. So **scopes already round-trip through
  the value model** and across a process boundary (`#wat.ast/ScopedSymbol {:name … :scopes […]}`,
  minted for execve). The bridge also records the corpus fact: only **macro-minted** symbols carry
  scopes; everything the parser emits and all hand-written code has an empty set.

## ★ What the flip does to that line — INFERENCE, marked as such

Today the line sits exactly where it should: **namespaced things need no hygiene** (the path
disambiguates — the `Keyword` doc says so), **bare local names do**. Colon-quoting is what puts
`:wat::core::+` on the no-hygiene side.

After the flip, `wat.core/+` is a **symbol**. One node kind would then carry both *"globally
unambiguous reference, hygiene meaningless"* and *"bare local name, hygiene load-bearing"* — the
fusion R28 exists to decomplect. If a macro ever mints `wat.core/+` with a scope set, it stops being
equal to a hand-written `wat.core/+` and resolution breaks unless something exempts namespaced
symbols.

**This consequence is the apparatus's reading of the two doc comments above, not a statement the
record makes and not a measured fact.** It is the thing to check first when 300.0 is scouted.

## The split that falls out — proposed; the builder rules

| | carries | per-occurrence state | internable |
|---|---|---|---|
| **namespaced** `wat.core/+` | a global path | none — the path disambiguates | **yes** |
| **bare** `x`, params, `let` binders | a local name | hygiene scopes | name yes, scopes alongside |
| **keyword** `:foo` (data) | itself | none | **yes** |

Clojure split keyword-vs-symbol on the **metadata** axis: `Symbol` holds a `final IPersistentMap
_meta`, so two `'x` with different metadata must be distinct objects and interning is impossible.
**We do not have that shape.** Our per-occurrence state is ONE closed structural field, so a third
option Clojure never had is available: **intern the name, keep the scopes alongside** —
`Arc<InternedName>` + `BTreeSet<ScopeId>`; equality is a pointer compare plus a set compare (the
common case being an empty set); hash is a cached name-hash combined with the scopes.

## The open question — the builder's, and it decides one type or two

> **When symbols become first-class VALUES, does a symbol value carry its scopes?**

The AST-level round trip already exists (`WatAST` values carry `Identifier`, scopes included). What
does not exist yet is a `Value` symbol variant — `:wat::core::i64` rides the keyword space today,
which is exactly what 109's note called out. So the question is not about the AST; it is about what a
*minted symbol value* is:

- **carries scopes** → symbols cannot be interned wholesale; keywords can; the split is Clojure's,
  reached by our own reason.
- **does not** → hygiene stays an AST concern, both kinds intern, one machinery.

Answer it from what a wat symbol **is**, not from what is convenient at drive time.

## The correctness content — this is why it is not optional

An identifier that is a `String` is manipulated with `format!` / `split` / `==` / `replace`, and arc
278 has been bitten by **that exact class four times**, one of which sits in this arc's path:

1. a companion name appended *past* `<T>` → `box-svc<T>::Record`
2. a type-ARG list flat-`split(',')`, tearing `State<K,V>` into `State<K` + `V>`
3. a `:messages` membership check comparing a base name against a declared `Name<K>`
4. **the `::`↔`.` dial is a `replace()`, not a parse** (`vocab.rs:225` / `edn_shim.rs:2783`, dated
   2026-05-21 arc 218) — filed in 278's 24u seam as *"load-bearing here: every keyword in an
   EDN-encoded AST goes through it"*, still unfixed

`CLAUDE.md` already carries the generalization: *"when a generic form misbehaves, suspect a string
comparison with one side normalized and the other not before suspecting the type system."*

Interning alone does not kill that class. **An identifier being a TYPE does** — `format!("{}::{}",
a, b)` stops compiling, and namespace-qualification goes through one door that gets made correct
once. Interning is then the obvious representation for that type, and hash-once and
identity-equality come free. **The type is the stone; interning is how it is stored.**

## The honest perf bound — do NOT sell this as a speed stone

109's note measured it (`binding_key_cost`, release, 50k iters, map held identical, key type varied):
lookup **1.0–1.1×**, build 1.2–1.7×. Its own closing line: *"Whoever picks this up should not inherit
a perf claim it cannot pay."*

Arc 278 added a second datum on 2026-08-01: `Token.bindings`' array-vs-trie lookup win of **3.2–3.5×**
was measured **with `Value::String` keys** — it is a *representation* win, not a *key* win, and does
not transfer. The case for this stone is **correctness and sequencing**, and it should be argued that
way or not at all.

## Proposed placement

**300.0 — the identifier becomes a type, interned — ahead of 300.1.** Not scouted, not briefed, no
size estimate; it is a value-model change reaching the reader, checker, runtime, EDN bridge and every
`format!` that builds a name, which is arc-shaped rather than stone-shaped. The method is proven
though: a type change gives a **compiler-enumerated** worklist rather than a grep (R52 `QVOD LEX
ACCENDIT`; 24r/24s ran that shape at 300+ files).

## Cross-references

- `docs/arc/2026/04/109-kill-std/NOTE-keyword-storage-must-intern.md` — the storage gap and the
  measured perf bound; this note is where its forcing function lands.
- `crates/wat-reader/src/identifier.rs:79` · `crates/wat-reader/src/ast.rs:105-113` — the two
  doc comments the inference above rests on.
- `src/runtime.rs:10089` / `:10470` — quote → `Value::wat__WatAST` → back.
- `src/wat_edn_bridge.rs:36` — `#wat.ast/ScopedSymbol`, and the only-macro-minted-symbols-carry-scopes
  fact.
- 278 `REALIZATIONS.md` far-side `24t` (a rename touches five surfaces) and `24u` (the `::`↔`.`
  `replace()`).
- This arc's `DESIGN.md` — 300.2–300.5, the drives this note asks to be sequenced behind.
