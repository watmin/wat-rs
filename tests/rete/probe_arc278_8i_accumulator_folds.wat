;; tests/rete/probe_arc278_8i_accumulator_folds.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :net::Packet record for accumulator fold tests.

(:wat::core::defrecord :net::Packet [src <- :wat::core::String])

;; The accumulators are PURE WAT FOLDS over a (PV :- [Element]). `els` = 3 Elements with bindings
;; {?bytes, ?port} + Packet facts: ?bytes = 100/200/300 (sum 600, min 100, max 300, mean 200);
;; ?port = 80/443/80 (distinct → 2; group-by → 2 keys). `empty` = an empty PV. Each entry point
;; wraps one accumulator call over `els` or `empty` — the ten probe assertions below.

;; count → BARE 3 (length is always concrete; never Option).
(:wat::core::defn :user::count-folds [] -> :wat::core::i64
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::rete::acc::count els)))

;; sum ?bytes → BARE 600 (empty sum = 0; never Option).
(:wat::core::defn :user::sum-folds [] -> :wat::core::i64
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::rete::acc::sum "?bytes" els)))

;; min ?bytes → Some(100).
(:wat::core::defn :user::min-folds [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::rete::acc::min "?bytes" els)))

;; max ?bytes → Some(300).
(:wat::core::defn :user::max-folds [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::rete::acc::max "?bytes" els)))

;; mean ?bytes → Some(200) — THE composition: sum(600)/count(3).
(:wat::core::defn :user::mean-is-sum-over-count [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::rete::acc::mean "?bytes" els)))

;; distinct ?port → BARE vec of length 2 (80, 443 — the duplicate 80 collapses).
(:wat::core::defn :user::distinct-folds [] -> :wat::core::i64
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::core::length (:wat::rete::acc::distinct "?port" els))))

;; all → BARE vec of length 3 (the gathered facts).
(:wat::core::defn :user::all-folds [] -> :wat::core::i64
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::core::length (:wat::rete::acc::all els))))

;; group-by ?port → BARE map with 2 keys (80 → [a,c], 443 → [b]).
(:wat::core::defn :user::group-by-folds [] -> :wat::core::i64
  (:wat::core::let
    [els (:wat::core::PersistentVector/conj
           (:wat::core::PersistentVector/conj
             (:wat::core::PersistentVector/conj (:wat::core::PersistentVector)
               (:wat::rete::Element :fact (:net::Packet :src "a") :bindings
                 (:wat::map::assoc
                   (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 100) "?port" 80)))
             (:wat::rete::Element :fact (:net::Packet :src "b") :bindings
               (:wat::map::assoc
                 (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 200) "?port" 443)))
           (:wat::rete::Element :fact (:net::Packet :src "c") :bindings
             (:wat::map::assoc
               (:wat::map::assoc (:wat::core::PersistentMap) "?bytes" 300) "?port" 80)))]
    (:wat::map::length (:wat::rete::acc::group-by "?port" els))))

;; EMPTY: count over an empty set → BARE 0 (count always concrete — never None).
(:wat::core::defn :user::count-empty-is-zero [] -> :wat::core::i64
  (:wat::core::let [empty (:wat::core::PersistentVector)]
    (:wat::rete::acc::count empty)))

;; EMPTY: min over an empty set → None (no token — there is no minimum of nothing).
(:wat::core::defn :user::min-empty-is-none [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let [empty (:wat::core::PersistentVector)]
    (:wat::rete::acc::min "?bytes" empty)))
