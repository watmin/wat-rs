;; wat-scripts/perf/grid/where-not-bound.wat — :not with a left-bound var (Clara
;; test-accum-result-in-negation). Empty-seed alpha cannot see `?v < ?m`; both
;; impls must re-match facts under the token. Mixed 50/40 → n=0; tied 50/50 → n=1.
;;
;; Twin of where-not-bound.clj.

(:wat::core::defn :wnb::row-count [] -> :wat::core::i64 2)

(:wat::core::defrecord :wnb::Station [loc <- :wat::core::String])
(:wat::core::defrecord :wnb::Reading [loc <- :wat::core::String v <- :wat::core::i64])
(:wat::core::defrecord :wnb::Busy    [loc <- :wat::core::String n <- :wat::core::i64])

(:wat::rete::defrule :wnb::max-not-below
  :when
  [(:wnb::Station (?loc <- :loc))
   (?m <- (:wat::rete::acc::max ?v) :from (:wnb::Reading (?loc <- :loc) (?v <- :v)))
   (:wat::rete::not (:wnb::Reading (?loc <- :loc) (?v <- :v)
                      (:wat::rete::i64::< ?v ?m)))]
  :then
  [(:wnb::Busy :loc ?loc :n ?m)])

(:wat::rete::defquery :wnb::q-Busy
  :params []
  :when [(?fact <- :wnb::Busy)])


(:wat::core::defn :wnb::fire [lo <- :wat::core::i64  hi <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::fire-rules
    (:wat::rete::insert
      (:wat::rete::compile-all (:wat::core::PersistentVector (:wnb::max-not-below)) (:wat::core::PersistentVector (:wnb::q-Busy)))
      (:wnb::Station :loc "OSL")
      (:wnb::Reading :loc "OSL" :v lo)
      (:wnb::Reading :loc "OSL" :v hi))))

(:wat::core::defn :wnb::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wnb::line 1 "max-not-below-mixed"
    (:wat::core::length (:wat::rete::query (:wnb::fire 50 40) (:wnb::q-Busy))))
  (:wnb::line 2 "max-not-below-tied"
    (:wat::core::length (:wat::rete::query (:wnb::fire 50 50) (:wnb::q-Busy)))))
