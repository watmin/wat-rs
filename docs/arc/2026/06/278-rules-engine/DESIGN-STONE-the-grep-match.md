# DESIGN — STONE: `:wat::grep::Match` (the fact a user's rule asserts)

> The Span fact (`5d650b807`) made the coordinate BINDABLE. This makes it REPORTABLE.
> Together they are the whole of wat-grep's data contract.

## The contract, restated

Builder, 2026-08-24: **the user's rules assert `Match` facts; wat-grep queries for them and prints.**
wat-grep owns exactly ONE query and performs NO interpretation. The user supplies RULES, not queries.
Everything wat-grep does not interpret is something it cannot get wrong.

Two consequences that shape everything below:
1. wat-grep never sees the match — only the fact the rule built. So every field must be **constructible
   inside a rete `:then`**, which is a much narrower surface than ordinary wat.
2. wat-grep never reads a field's meaning. So the fields must be legible to an **arbitrary** consumer,
   not just to `wat/fix.wat`.

## The record

```clojure
(:wat::core::defrecord :wat::grep::Binding
  [name  <- :wat::core::String      ; what the rule called it — "id", "kind", "head"
   value <- :wat::core::String])    ; what it bound, rendered

(:wat::core::defrecord :wat::grep::Match
  [file      <- :wat::core::String  ; supplied by the RUN, not by a node
   line      <- :wat::core::i64
   col       <- :wat::core::i64
   end-line  <- :wat::core::i64     ; REQUIRED — see "the end is not optional"
   end-col   <- :wat::core::i64
   rule      <- :wat::core::String  ; which rule concluded this
   bindings  <- (:wat::core::PersistentVector :- [:wat::grep::Binding])])
```

Flat coordinates, not a nested span. Same reason `:fx::Span` is flat: **a rule binds FIELDS, not
sub-records**, and a downstream rule that wants to reason about a Match's line must be able to write
`(:wat::grep::Match (?l <- :line))` without destructuring first.

## The end is NOT optional — and that is a REJECTION of the substrate's own Span

Builder, 2026-08-24: *"i don't care if wat-grep's Match imposes a hard requirement on end coords,
that's better imo.. but the data provider /is an option/ so we need to process it correctly."*

`:wat::core::Span` carries `end <- (:wat::core::Option :- [:wat::core::Pos])`, and the substrate says
why in its own hand (`crates/wat-reader/src/span.rs:69`): *"`end` is `Some(Pos)` when the lexer or
parser computed a real range; `None` for point-spans from Rust call sites (`rust_caller_span!()`)
where no end is available."* Two constructors split on exactly that line — `Span::new` for Rust,
`Span::with_end` for the lexer and parser.

**That Option exists for RUST's benefit.** `ast-span` and `ast-end-span` are TOTAL — measured on the
Span stone across every leaf kind AND reader-synthesized nodes (`sigils-inline Node=23 Span=23`).
wat always knows its end. A `Match` that reused `:wat::core::Span` would inherit a `None` case wat
cannot produce, and every consumer forever would unwrap a variant that is never there.

★ **And the shape pays a second dividend that is worth naming, because it is the difference between
avoiding a problem and not having one.** With no Option field on `Match`, **a user's rule never
constructs `Some`** — so the bare-vs-declared variant spelling that cost this session an hour
(`:wat::core::Some` refused in a `:then`; `:wat::core::Option::Some` admitted) never touches wat-grep
at all. Not worked around. Structurally absent.

## `bindings` is a VECTOR, not a map — and this is MEASURED, not preferred

`294/SEAM.md` sketched `bindings <- PersistentMap`. **A rete `:then` cannot build a map.** Measured
this session, at `compile-all` time:

```
(:wat::core::PersistentMap ?k ?v)         compile-condition: then expr is not total
                                          — ':wat::core::PersistentMap' is not total
(:wat::rete::core::PersistentMap ?k ?v)   passes the fence, then dies at runtime:
                                          "compiled apply cannot dispatch kind Unknown arity 2"
```

The rete-vocabulary spelling has a `RETE_OPS` row and NO compiled-exec implementation — it clears the
axis fence and has nothing to run. (That gap is rete's, i.e. grok-rete's; it is NOT this stone's to
close, and this stone does not need it.)

What a `:then` CAN construct, all measured:

```
bound ?var                              ✓   {:x "?id"}
string / int / float / bool literal     ✓   {:x "lit"}
:wat::rete::core::String/concat ?k ?v   ✓   {:x "?id42"}
:wat::rete::core::i64::+ ?n 1           ✓   {:x 8}
:wat::rete::core::PersistentVector …    ✓   {:x #wat.core/PersistentVector ["?id" "42"]}
a declared record constructor           ✓   (Law A — declaration-derived heads are admitted)
a declared enum variant constructor     ✓   (:g::End::Known ?l ?c) -> #g.End/Known [7 26]
:wat::core::string::concat              ✗   is not total
a bare keyword variant (:wat::core::None) ✗ RhsUnresolvableOperand
```

So the vector-of-records IS buildable, whole, inside one RHS, with LHS bindings flowing into the
nested constructors — measured verbatim:

```
#m/Out {:x #wat.core/PersistentVector [#m/Binding {:name "id"   :value "?id"}
                                        #m/Binding {:name "kind" :value "symbol"}]}
```

⚠ Note the constructor is **`:wat::rete::core::PersistentVector`**, not core's. A rule author writing
core's spelling gets *"is not total"*, which describes the axis, not the fix.

### Why `Binding` is a record and not two parallel vectors

A `(PersistentVector :- [String])` of alternating name/value would build just as well and is strictly
worse: it is a map with the pairing left to a convention nobody can check, and an odd-length vector
would be a silent defect that renders fine. The record makes the pair the unit.

## The one door where an Option is genuinely processed

There are exactly TWO Options in wat-grep's path. The design above deletes the second. The first is
real and must be handled:

```
ast-span node            → (HashMap :- [keyword i64])   {:line N :col C}   no Option
ast-end-span node        → same                                            no Option
HashMap/get span :line   → Option<i64>                        ← ① THE ONE
:wat::core::Span/end     → Option<Pos>                        ← ② deleted by the shape above
```

Today the extractor unwraps ① **four times per node** with a separate `Option/expect` and its own
message string. That is the CONVENTION rung: four sites, four chances to write a different message or
forget one, on a value fetched from a HashMap the caller did not build.

**This stone collapses it to one door**: a single verb that takes a `:wat::WatAST` and yields the four
coordinates, consuming the `HashMap/get` Options exactly once, with one located failure. No other site
sees an Option, and no rule can — because the fact it binds is already four i64s.

```clojure
(:wat::core::defn :wat::grep::coords-of [node <- :wat::WatAST] -> :wat::grep::Coords …)
```

The extractor's Span emit calls it; nothing else unwraps a span.

## The rooms

1. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat:89-99`** — the four `Option/expect`
   calls the one door replaces.
2. **`wat-scripts/lib/wat-grep.wat`** — 93 lines, TOP-LEVEL ONLY, and it already computes both spans
   and throws them away (`wat-grep-form-edit:37-38`). This is where `Match` gets queried and printed.
3. **`wat/fix.wat:179-193`** — `fix-text-offset-of`, the canonical `(Option/expect (HashMap/get …))`
   chain the one door absorbs; and `fix-text-offset-of` is also why `Match` carries NO `:offset`
   (`fix.wat` derives it from `{:line :col}` + the file's lines; carrying both gives one position two
   sources of truth that drift the moment anything re-reads the file).

## Acceptance

1. **A rule builds a complete `Match` in one RHS** — all five coordinates LHS-bound from `:fx::Span`,
   `file` supplied as a literal, `bindings` a non-empty vector of `Binding` records. Output verbatim.
2. **No `Option` appears anywhere in the Match's rendered EDN.** The negative control for the whole
   "end is not optional" ruling — grep the output for `Option/` and find nothing.
3. **`coords-of` is the ONLY site that unwraps an `ast-span` HashMap.** A census of
   `Option/expect` + `HashMap/get` in the touched files returns exactly one pair.
4. **The one door is total on the same population the Span stone measured** — re-run corpus-03 and get
   `Span == Node` unchanged (4316 / 435 / 33). A refactor that changes a count changed behaviour.
5. Files load (`every_wat_scripts_file_loads`), floor green, clippy 0.

## Out of scope — affirmatively cut

- **The rete PersistentMap compiled-exec gap.** Measured and recorded above; it belongs to the rete
  subsystem, which is grok-rete's authority. This stone routes around it by design, not by workaround
  — the vector is the better shape regardless.
- **wat-grep's loop, the network, and `with-network`.** The Match record and its one door are this
  stone; the processor that runs files through a held network is the next.
- **Walking deep.** wat-grep is TOP-LEVEL ONLY today; `fix.wat`'s `fix-source` has the recursion. That
  is a separate change with its own blast radius.
- **`:wat::core::Span` / the Option/Result symbol migration.** Belongs to `296/DESIGN-STONE-H`, which
  is DRAWN and carries it explicitly. Nothing here waits on it.

## ⚠ OWED BEFORE SHIPPING — cast `intueri`

`Match` is the builder's name. `Binding`, `Coords`, and `coords-of` are the apparatus's and have not
been ruled. intueri owns all names; cast it on this file's three new nouns before the stone lands,
the same way the scoped-work names were ruled (`61b07ccc3`).
