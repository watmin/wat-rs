;; tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat — comparison-gate world for the where_oracle probe;
;; loaded via startup_from_file. Rule filters Temperature by (where (> ?c 0)).

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wg::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wg::cold-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::core::> ?c 0))]
  :then
  [(:wg::Gate :celsius ?c)])

;; 1 — the where PASSES: Temp(5), (> 5 0) true → exactly one Gate derived.
(:wat::core::defn :user::run-gate-c5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wg)
       session (:wat::rete::compile rules)
       session (:wat::rete::insert session (:weather::Temperature :celsius 5 :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query-by-type-string fired "wg::Gate"))))

;; 2 — the where BLOCKS: Temp(-5), (> -5 0) false → zero Gates (the filter actually filters).
(:wat::core::defn :user::run-gate-cneg5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wg)
       session (:wat::rete::compile rules)
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query-by-type-string fired "wg::Gate"))))

