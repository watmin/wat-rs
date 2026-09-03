# WORKLIST — the 121 call heads the registry cannot vouch for

> Derived 2026-09-01 by EXPERIMENT, not by reading. The procedure, so the number can be
> re-derived rather than trusted (`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`):
>
> 1. In `src/resolve/walk.rs`, `is_resolvable_call_head`, replace the short-circuit
>    `if is_reserved_prefix(head) { return true; }` with
>    `if is_reserved_prefix(head) { if registry().lookup_entry(head).is_some() { return true; } }`
> 2. `cargo build --release --bin wat`
> 3. `for f in $(find wat wat-scripts -name "*.wat"); do ./target/release/wat --check "$f" 2>&1; done \
>      | grep -o ":path \"[^\"]*\"" | sed "s/:path \"//;s/\"//" | sort | uniq -c | sort -rn`
> 4. REVERT the patch.

At the time of measurement: **578 of 599** corpus files failed; **121** distinct names.
Of them: **0** are registry rows, **68** have a checker `TypeScheme` but no registry row,
**53** are known by neither.

## The names, by corpus call-site count

```
   3952 :wat::core::fn
   3431 :wat::core::def
   1613 :wat::core::match
    873 :wat::core::quote
    687 :wat::core::=
    603 :wat::core::do
    483 :wat::core::PersistentVector
    380 :wat::core::foldl
    362 :wat::core::first
    330 :wat::eval-ast!
    271 :wat::core::Tuple
    244 :wat::core::ann-form
    230 :wat::core::second
    222 :wat::core::PersistentMap
    169 :wat::core::get
    157 :wat::core::extend-type
    135 :wat::core::str
    133 :wat::core::forms
    121 :wat::core::quasiquote
    111 :wat::rete::string::=
     78 :wat::rete::i64::>
     65 :wat::core::map
     51 :wat::core::<
     47 :wat::core::derive
     34 :wat::rete::i64::+
     33 :wat::rete::core::and
     32 :wat::rete::string::starts-with?
     32 :wat::rete::i64::=
     32 :wat::core::>
     28 :wat::core::bool::to-string
     26 :wat::core::apply
     25 :wat::core::>=
     23 :wat::rete::i64::<
     20 :wat::core::show
     19 :wat::core::or
     18 :wat::rete::core::if
     17 :wat::rete::i64::*
     16 :wat::rete::core::or
     15 :wat::stream::lazy
     15 :wat::rete::i64::/
     14 :wat::rete::core::not
     14 :wat::core::not
     13 :wat::rete::vector::get
     13 :wat::core::macroexpand
     11 :wat::rete::i64::-
     10 :wat::rete::vector::length
     10 :wat::rete::i64::mod
     10 :wat::core::not=
      9 :wat::type::Tuple
      9 :wat::core::filter
      8 :wat::rete::string::contains?
      8 :wat::rete::map::contains-key?
      8 :wat::rete::holon::cosine
      8 :wat::rete::core::foldl
      8 :wat::core::u8
      8 :wat::core::defclause
      7 :wat::type::i64
      7 :wat::rete::f64::>
      7 :wat::core::contains?
      7 :wat::core::and
      6 :wat::rete::vector::contains?
      6 :wat::rete::string::length
      6 :wat::rete::core::let
      6 :wat::rete::core::fn
      6 :wat::rete::core::PersistentVector/first
      6 :wat::core::stream->vec
      6 :wat::core::<=
      5 :wat::type::String
      5 :wat::rete::string::subs
      5 :wat::rete::i64::not=
      5 :wat::rete::f64::/
      5 :wat::rete::f64::*
      5 :wat::core::third
      4 :wat::rete::i64::>=
      4 :wat::rete::core::match
      4 :wat::rete::core::keyword::=
      3 :wat::rete::i64::to-f64
      3 :wat::rete::core::enum::=
      3 :wat::eval-with-defs!
      3 :wat::core::None
      2 :wat::type::Vector
      2 :wat::rete::vec::get
      2 :wat::rete::string::trim
      2 :wat::rete::string::to-lowercase
      2 :wat::rete::string::ends-with?
      2 :wat::rete::string::empty?
      2 :wat::rete::string::concat
      2 :wat::rete::linkedlist::get
      2 :wat::rete::i64::rem
      2 :wat::rete::i64::<=
      2 :wat::rete::holon::dot
      2 :wat::rete::f64::<
      2 :wat::rete::core::enum::not=
      2 :wat::rete::core::Vector/first
      2 :wat::rete::core::PersistentVector
      2 :wat::rete::core::List/first
      2 :wat::core::println
      2 :wat::core::mapv
      2 :wat::core::edn::write
      1 :wat::spawn::process/grants
      1 :wat::rete::string::not=
      1 :wat::rete::i64::to-string
      1 :wat::rete::i64::quot
      1 :wat::rete::holon::presence?
      1 :wat::rete::holon::coincident?
      1 :wat::rete::f64::to-string
      1 :wat::rete::f64::not=
      1 :wat::rete::f64::>X
      1 :wat::rete::f64::>=
      1 :wat::rete::f64::=
      1 :wat::rete::f64::<=
      1 :wat::rete::f64::+
      1 :wat::rete::core::reduce
      1 :wat::rete::core::map
      1 :wat::rete::core::filter
      1 :wat::rete::core::bool::to-string
      1 :wat::core::tuple-get
      1 :wat::core::reduce-walk
      1 :wat::core::macroexpand-1
      1 :wat::core::find-last-index
      1 :wat::core::conforms?
```

---

## ⛔ RE-DERIVED 2026-09-02 — **121 → 107**, and the answer to *"are we ready to rip out the whitelist?"* is NO

The procedure at the top was re-run in full (patch → build → sweep 609 corpus files → revert):

```
                      2026-09-01        2026-09-02
corpus files              599               609
failing                   578               509
distinct names            121               107
```

★ The campaign has moved it by **14 names** while the corpus grew by 10 files. **107 names still
have no registry row**, so flipping `is_resolvable_call_head` today still fails **509 of 609**
files. The order the RULING forces has not changed: registry answers → consumer asks → duplicate
dies.

## ★★★ Where the remaining 107 live — this is the road map, measured

```
:wat::rete::*            66     ← RETE_OPS' population. PHASE 1b. The single largest block.
   core 19 · i64 15 · string 11 · f64 11 · holon 4 · vector 3 · map/vec/linkedlist 3
:wat::core::             33     ← two DIFFERENT populations mixed:
                                  · the 1a-ζ remainder — do · ann-form (+ extend-type, derive,
                                    defclause, which are declare/parse's 6-name hand-list)
                                  · collection/arithmetic verbs — = · first · second · get · map ·
                                    str · foldl · < · > · apply · PersistentVector · PersistentMap ·
                                    Tuple — NOT in special_forms.rs at all; these are GAP_B's own
                                    population and need their own stones
:wat::type               4      · Tuple · i64 · String · Vector
misc                     4      · eval-ast! · eval-with-defs! · stream::lazy · spawn::process/grants
```

★★ **Phase 1b is worth 66 of the 107 on its own** — more than half, in one table. And it is no
longer blocked: its stated blocker was *"until and/or/cond's targets are registered"*, and `and`/`or`
were registered at Stone 1a-i.

⚠ **The `:wat::core::` 33 are not one job.** Roughly a third are forms `special_forms.rs` knows about
(1a-ζ's remainder, plus `declare`'s own hand-list population); the rest are ordinary collection and
arithmetic verbs that were never special forms and are not on any table this campaign has yet
attacked. **Counting them as one number is how a plan gets written that cannot be executed.**

## The full 107, by corpus call-site count

```
688 :wat::core::=
    609 :wat::core::do
    483 :wat::core::PersistentVector
    380 :wat::core::foldl
    362 :wat::core::first
    330 :wat::eval-ast!
    271 :wat::core::Tuple
    244 :wat::core::ann-form
    230 :wat::core::second
    222 :wat::core::PersistentMap
    169 :wat::core::get
    157 :wat::core::extend-type
    135 :wat::core::str
    111 :wat::rete::string::=
     78 :wat::rete::i64::>
     65 :wat::core::map
     51 :wat::core::<
     47 :wat::core::derive
     34 :wat::rete::i64::+
     33 :wat::rete::core::and
     32 :wat::rete::string::starts-with?
     32 :wat::rete::i64::=
     32 :wat::core::>
     26 :wat::core::apply
     25 :wat::core::>=
     23 :wat::rete::i64::<
     19 :wat::rete::core::if
     17 :wat::rete::i64::*
     16 :wat::rete::core::or
     15 :wat::stream::lazy
     15 :wat::rete::i64::/
     14 :wat::rete::core::not
     13 :wat::rete::vector::get
     11 :wat::rete::i64::-
     10 :wat::rete::vector::length
     10 :wat::rete::i64::mod
     10 :wat::core::not=
      9 :wat::type::Tuple
      9 :wat::core::filter
      8 :wat::rete::string::contains?
      8 :wat::rete::map::contains-key?
      8 :wat::rete::holon::cosine
      8 :wat::rete::core::foldl
      8 :wat::core::defclause
      7 :wat::type::i64
      7 :wat::rete::f64::>
      7 :wat::core::contains?
      6 :wat::rete::vector::contains?
      6 :wat::rete::string::length
      6 :wat::rete::core::let
      6 :wat::rete::core::fn
      6 :wat::rete::core::PersistentVector/first
      6 :wat::core::stream->vec
      6 :wat::core::<=
      5 :wat::type::String
      5 :wat::rete::string::subs
      5 :wat::rete::i64::not=
      5 :wat::rete::f64::/
      5 :wat::rete::f64::*
      5 :wat::core::third
      4 :wat::rete::i64::>=
      4 :wat::rete::core::match
      4 :wat::rete::core::keyword::=
      3 :wat::rete::i64::to-f64
      3 :wat::rete::core::enum::=
      3 :wat::eval-with-defs!
      3 :wat::core::None
      2 :wat::type::Vector
      2 :wat::rete::vec::get
      2 :wat::rete::string::trim
      2 :wat::rete::string::to-lowercase
      2 :wat::rete::string::ends-with?
      2 :wat::rete::string::empty?
      2 :wat::rete::string::concat
      2 :wat::rete::linkedlist::get
      2 :wat::rete::i64::rem
      2 :wat::rete::i64::<=
      2 :wat::rete::holon::dot
      2 :wat::rete::f64::<
      2 :wat::rete::core::enum::not=
      2 :wat::rete::core::Vector/first
      2 :wat::rete::core::PersistentVector
      2 :wat::rete::core::List/first
      2 :wat::core::println
      2 :wat::core::mapv
      2 :wat::core::edn::write
      1 :wat::spawn::process/grants
      1 :wat::rete::string::not=
      1 :wat::rete::i64::to-string
      1 :wat::rete::i64::quot
      1 :wat::rete::holon::presence?
      1 :wat::rete::holon::coincident?
      1 :wat::rete::f64::to-string
      1 :wat::rete::f64::not=
      1 :wat::rete::f64::>X
      1 :wat::rete::f64::>=
      1 :wat::rete::f64::=
      1 :wat::rete::f64::<=
      1 :wat::rete::f64::+
      1 :wat::rete::core::reduce
      1 :wat::rete::core::map
      1 :wat::rete::core::filter
      1 :wat::rete::core::bool::to-string
      1 :wat::core::tuple-get
      1 :wat::core::reduce-walk
      1 :wat::core::find-last-index
      1 :wat::core::conforms?
```


---

## ⛔ RE-DERIVED 2026-09-03 — **107 → 71**, after Stones 1b-i and 1b-ii

The procedure at the top of this file was re-run in full (patch → build → sweep → **revert**,
diff verified empty):

```
                      2026-09-01        2026-09-02        2026-09-03
corpus files              599               609               610
failing                   578               509               505
distinct names            121               107                71
```

The 36 that left are exactly the two alias stones: 28 `OpClass::Alias` rows (1b-i) and 8
`Form`/`Redispatch` rows (1b-ii). `107 − 36 = 71`, with no surprises in either direction —
the first phase of this campaign whose corpus effect was predicted exactly before it ran.

## ★★★ THE ROAD MAP INVERTS WHEN YOU WEIGH IT BY USE, NOT BY NAME

```
                names   call sites   sites/name
:wat::core::       33        4,260        129     ← 87% of ALL remaining unresolvable calls
:wat::rete::       30          278          9
:wat::type          4           23          6
misc                4          349         87     ← eval-ast! alone is 330
                   ──        ─────
                   71        4,910
```

⚠ **The rete remainder is 42% of the NAMES and 6% of the USE.** Reading this worklist as a
name-count — which is how it has been read all campaign, including in the SEAM's own NEXT —
puts the cheapest half of the corpus's exposure at the top of the list. A count is not a share.

★ And the single biggest lever is not a family at all: **`:wat::core::do` (609 sites) and
`:wat::core::ann-form` (244) are two of the three names left in Phase 1a-ζ** — the last unfinished
family of the `special_forms.rs` 35. Three rows, 868 call sites, already scoped.

## The full 71, by corpus call-site count

```
    688 :wat::core::=
    609 :wat::core::do
    483 :wat::core::PersistentVector
    380 :wat::core::foldl
    362 :wat::core::first
    330 :wat::eval-ast!
    271 :wat::core::Tuple
    244 :wat::core::ann-form
    230 :wat::core::second
    222 :wat::core::PersistentMap
    169 :wat::core::get
    157 :wat::core::extend-type
    135 :wat::core::str
    111 :wat::rete::string::=
     65 :wat::core::map
     51 :wat::core::<
     47 :wat::core::derive
     34 :wat::rete::i64::+
     32 :wat::core::>
     26 :wat::core::apply
     25 :wat::core::>=
     17 :wat::rete::i64::*
     15 :wat::stream::lazy
     15 :wat::rete::i64::/
     13 :wat::rete::vector::get
     11 :wat::rete::i64::-
     10 :wat::rete::i64::mod
     10 :wat::core::not=
      9 :wat::type::Tuple
      9 :wat::core::filter
      8 :wat::rete::holon::cosine
      8 :wat::rete::core::foldl
      8 :wat::core::defclause
      7 :wat::type::i64
      7 :wat::core::contains?
      6 :wat::rete::core::PersistentVector/first
      6 :wat::core::stream->vec
      6 :wat::core::<=
      5 :wat::type::String
      5 :wat::rete::string::subs
      5 :wat::rete::f64::/
      5 :wat::rete::f64::*
      5 :wat::core::third
      4 :wat::rete::core::keyword::=
      3 :wat::rete::core::enum::=
      3 :wat::eval-with-defs!
      3 :wat::core::None
      2 :wat::type::Vector
      2 :wat::rete::vec::get
      2 :wat::rete::linkedlist::get
      2 :wat::rete::i64::rem
      2 :wat::rete::holon::dot
      2 :wat::rete::core::enum::not=
      2 :wat::rete::core::Vector/first
      2 :wat::rete::core::PersistentVector
      2 :wat::rete::core::List/first
      2 :wat::core::println
      2 :wat::core::mapv
      2 :wat::core::edn::write
      1 :wat::spawn::process/grants
      1 :wat::rete::string::not=
      1 :wat::rete::i64::quot
      1 :wat::rete::f64::>X
      1 :wat::rete::f64::+
      1 :wat::rete::core::reduce
      1 :wat::rete::core::map
      1 :wat::rete::core::filter
      1 :wat::core::tuple-get
      1 :wat::core::reduce-walk
      1 :wat::core::find-last-index
      1 :wat::core::conforms?
```
