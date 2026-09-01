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
