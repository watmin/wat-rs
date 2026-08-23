;; tests/rete/probe_arc278_then_user_forms_userfn.wat — Stone B widening (a) GREEN world.
;; Loaded via startup_from_file. The `:then` item's HEAD is a user fn (`:tf::first-rate`), not a
;; fact-type constructor — admissible because its declared return type (`:tf::Rate`) is a fact
;; type and its body bottoms out in admitted ops (`:wat::core::first` on a bound
;; `PersistentVector<tf::Rate>` — the accumulator's `acc::all` result, which BEFORE this stone
;; `:then` had no way to consume at all: "the rete action layer only inserts records from bound
;; vars + literals" — accum.wat's own comment). This is new capability, not merely a widened
;; syntax: a `where` fence never had to prove a fn's RETURN type; that check is `:then`-only.
;;
;; ★ WHY EXTRACTION, NOT CONSTRUCTION: `:tf::first-rate` selects an EXISTING accumulated fact
;; rather than building a new one. That is not a stylistic choice — it works AROUND a separate,
;; pre-existing, already-tracked substrate gap: `purity.rs`'s `KNOWN_UNREVIEWED` ratchet lists
;; `:wat::core::kwargs-construct` AND `:wat::core::aggregate-new` as genuinely unclassified (its
;; own comment: "these 215 are genuinely unruled... this list IS the debt, by name"). Every
;; surface a user fn can build a NEW aggregate through (kwargs sugar, positional sugar, even the
;; type's own PRIME constructor) macro-expands to one of those two heads, so a composed fn whose
;; body constructs a record is refused today with "`:wat::core::kwargs-construct` is not pure" —
;; the identical fence any `where`-fn hits, just never previously EXERCISED because a `where`
;; predicate never had to return one. This is unrelated to and unfixable by Stone B's own fence
;; (`purity.rs` is explicitly out of scope, per BRIEF-then-user-forms.md's read list); it is
;; reported here rather than routed around.

(:wat::core::defrecord :tf::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :tf::Rate   [count <- :wat::core::i64])

(:wat::rete::core::defn :tf::first-rate
  [rs <- (:wat::core::PersistentVector :- [:tf::Rate])]
  -> :tf::Rate
  (:wat::rete::core::PersistentVector/first rs :undefined (:tf::Rate :count 0)))

(:wat::rete::defrule :tf::gather
  :when [(:tf::Anchor (?x <- :x))
         (?rates <- (:wat::rete::acc::all) :from (:tf::Rate (?c <- :count)))]
  :then [(:tf::first-rate ?rates)])

(:wat::rete::defquery :tf::q-Rate
  :params []
  :when [(:tf::Rate (?count <- :count))])


;; Fires via the WAT ORACLE. NOT an unconfounded witness for "a NEW fact was derived" — the
;; extraction-only fn returns a value structurally IDENTICAL to the accumulated input, so a plain
;; type-count cannot distinguish "the rule fired" from "the input fact was already there" (both
;; read back as one `tf::Rate`, same `count`). What this DOES prove, unconfounded: the fence
;; admits a fn-headed item whose body reads a `PersistentVector<Record>`-valued accumulate bind
;; (impossible for `:then` before this stone), `sym.functions` resolution + `apply_function`
;; execute it, and the result type-checks as a fact at `build_insert_fact_call`'s runtime guard —
;; all without raising. See `probe_arc278_then_user_forms.rs` for what's actually asserted.
(:wat::core::defn :user::run-first-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :tf)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:tf::q-Rate)))
     session (:wat::rete::insert session (:tf::Anchor :x 0))
     session (:wat::rete::insert session (:tf::Rate :count 5))
     fired   (:wat::rete::fire-rules$oracle session)
     derived (:wat::rete::query fired (:tf::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::core::PersistentMap/get r "?count")
      "q-Rate: ?count")))

(:wat::core::defn :user::run-first-count-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :tf)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:tf::q-Rate)))
     session (:wat::rete::insert session (:tf::Anchor :x 0))
     session (:wat::rete::insert session (:tf::Rate :count 5))
     fired   (:wat::rete::fire-rules session)
     derived (:wat::rete::query fired (:tf::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::core::PersistentMap/get r "?count")
      "q-Rate: ?count")))
