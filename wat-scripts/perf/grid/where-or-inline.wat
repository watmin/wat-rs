;; wat-scripts/perf/grid/where-or-inline.wat — an `:or` / `:not` nested INSIDE one
;; condition's constraint list. Twin of where-or-inline.clj.
;;
;; WHY THIS FAMILY EXISTS. There are THREE different `or`s in this engine and the
;; corpus only had two of them:
;;   1. TOP-LEVEL, across conditions — `:when [(:wat::rete::or (:A …) (:B …))]`.
;;      Compiles to network branches. Covered by where-or-conditions / where-or-and.
;;   2. A WHERE-EXPRESSION — `(:wat::rete::where (:wat::rete::core::or ?a ?b))`.
;;      Covered by where-boolean.
;;   3. INTRA-CONDITION — an `:or` in ONE pattern's constraint list, as below. This
;;      is the only shape that compiles to `compiled_cond`'s `Op::Or` / `Op::Not`
;;      (`compile_one`, the `ReteClauseShape::Or` / `::Not` arms), and NOTHING in
;;      the corpus wrote it.
;;
;; That was proved, not assumed: on 2026-08-24 a `panic!` was armed in both arms and
;; the ENTIRE where-family differential passed, as did a direct run of
;; where-boolean.wat. The arms are live and reachable — the same panic fires
;; immediately on the shape below — they simply had zero coverage in any of the
;; three implementations. `compiled_cond.rs`'s own doc says the arms are "not
;; exercised by anything in the live grid corpus"; the truth was broader.
;;
;; The `or`/`not` branches here run over a THROWAWAY scope clone (`compile_one`'s
;; own comment: "for or/not branches the caller passes a throwaway clone/scratch …
;; matching eval_clause's discard of branch-local binds"). Row 4 is the one that
;; would catch a leak: `?hi` is bound only inside an `:or` arm, so it must NOT be
;; visible to the RHS — the rule binds `?k` from the pattern and nothing else.
;;
;; n= is DISTINCT keys, so a duplicate token cannot inflate a row.

(:wat::core::defrecord :woi::Reading [k <- :wat::core::i64  v <- :wat::core::i64  loc <- :wat::core::String])
(:wat::core::defrecord :woi::Station [loc <- :wat::core::String])
(:wat::core::defrecord :woi::Hit [k <- :wat::core::i64])
(:wat::core::defrecord :woi::At  [loc <- :wat::core::String])

;; ROW 1-2 — inline :or of two constraints on the SAME pattern. Extreme reading.
(:wat::rete::defrule :woi::extreme
  :when [(:woi::Reading (?k <- :k) (?v <- :v)
           (:wat::rete::or
             (:wat::rete::core::i64::> ?v 30)
             (:wat::rete::core::i64::< ?v 10)))]
  :then [(:woi::Hit :k ?k)])

;; ROW 3 — inline :not of a single constraint. NOT extreme-high.
(:wat::rete::defrule :woi::not-high
  :when [(:woi::Reading (?k <- :k) (?v <- :v)
           (:wat::rete::not (:wat::rete::core::i64::> ?v 30)))]
  :then [(:woi::Hit :k ?k)])

;; ROW 4 — De Morgan: inline :not of an inline :or. Also the SCOPE-LEAK row: ?hi is
;; bound only inside an :or arm and must not escape to the RHS.
(:wat::rete::defrule :woi::mid
  :when [(:woi::Reading (?k <- :k) (?v <- :v)
           (:wat::rete::not
             (:wat::rete::or
               (:wat::rete::core::i64::> ?v 30)
               (:wat::rete::core::i64::< ?v 10))))]
  :then [(:woi::Hit :k ?k)])

;; ROW 5 — the same inline :or behind a join prefix, so the branch scope is cloned
;; from a scope that already carries ?loc rather than from an empty one.
(:wat::rete::defrule :woi::station-extreme
  :when [(:woi::Station (?loc <- :loc))
         (:woi::Reading (?loc <- :loc) (?k <- :k) (?v <- :v)
           (:wat::rete::or
             (:wat::rete::core::i64::> ?v 30)
             (:wat::rete::core::i64::< ?v 10)))]
  :then [(:woi::At :loc ?loc)])

(:wat::rete::defquery :woi::q-Hit :params [] :when [(?fact <- :woi::Hit)])
(:wat::rete::defquery :woi::q-At  :params [] :when [(?fact <- :woi::At)])

(:wat::core::defn :woi::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:woi::q-Hit))))
(:wat::core::defn :woi::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:woi::q-At))))

(:wat::core::defn :woi::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [qs   (:wat::core::PersistentVector (:woi::q-Hit) (:woi::q-At))
                    ext  (:wat::core::PersistentVector (:woi::extreme))
                    nhi  (:wat::core::PersistentVector (:woi::not-high))
                    mid  (:wat::core::PersistentVector (:woi::mid))
                    pref (:wat::core::PersistentVector (:woi::station-extreme))]
    (:woi::line 1 "or-hits-high"
      (:woi::n-hit (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert-all (:wat::rete::compile-all ext qs)
        (:wat::core::PersistentVector (:woi::Reading :k 1 :v 40 :loc "MCI") (:woi::Reading :k 2 :v 20 :loc "MCI"))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woi::line 2 "or-hits-low"
      (:woi::n-hit (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert-all (:wat::rete::compile-all ext qs)
        (:wat::core::PersistentVector (:woi::Reading :k 1 :v 5 :loc "MCI") (:woi::Reading :k 2 :v 20 :loc "MCI"))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woi::line 3 "not-high"
      (:woi::n-hit (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert-all (:wat::rete::compile-all nhi qs)
        (:wat::core::PersistentVector (:woi::Reading :k 1 :v 40 :loc "MCI") (:woi::Reading :k 2 :v 20 :loc "MCI"))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woi::line 4 "demorgan-mid"
      (:woi::n-hit (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert-all (:wat::rete::compile-all mid qs)
        (:wat::core::PersistentVector (:woi::Reading :k 1 :v 40 :loc "MCI") (:woi::Reading :k 2 :v 20 :loc "MCI") (:woi::Reading :k 3 :v 5 :loc "MCI"))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:woi::line 5 "prefix-or"
      (:woi::n-at (:wat::core::match (:wat::rete::fire-rules
        (:wat::core::match (:wat::rete::insert-all
          (:wat::core::match (:wat::rete::insert-all (:wat::rete::compile-all pref qs)
            (:wat::core::PersistentVector (:woi::Station :loc "MCI"))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
          (:wat::core::PersistentVector (:woi::Reading :k 1 :v 40 :loc "MCI"))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
