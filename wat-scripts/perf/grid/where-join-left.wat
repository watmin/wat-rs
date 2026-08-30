;; wat-scripts/perf/grid/where-join-left.wat — HashJoin whose RIGHT cond names a
;; LEFT-bound var inline (`?w > ?c`). Twin of where-join-left.clj.
;;
;; Clara 0.24.0 evaluates `(> ?w ?c)` at beta after both facts bind.
;; Wat HashJoin only checks shared-var agreement (`?loc`). Empty-seed alpha
;; cannot see `?c`; local populate then drops the compare. Either way this
;; family is the missing expressivity cell: same predicate as a trailing
;; `:where` (rows 7–8, both engines), spelled on the join (rows 4–6, 9).
;;
;; Prove the miss before fixing the join rematch.

(:wat::core::defrecord :wjl::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wjl::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wjl::Hit  [loc <- :wat::core::String])

;; THE HOLE — leftover `?c` lives on the Wind cond, not in a :where.
(:wat::rete::defrule :wjl::wind-above-temp-inline
  :when
  [(:wjl::Temp (?loc <- :loc) (?c <- :c))
   (:wjl::Wind (?loc <- :loc) (?w <- :kph)
     (:wat::rete::core::i64::> ?w ?c))]
  :then
  [(:wjl::Hit :loc ?loc)])

;; CONTROL — same predicate, TestNode after the join. Both engines honor this.
(:wat::rete::defrule :wjl::wind-above-temp-where
  :when
  [(:wjl::Temp (?loc <- :loc) (?c <- :c))
   (:wjl::Wind (?loc <- :loc) (?w <- :kph))
   (:wat::rete::where (:wat::rete::core::i64::> ?w ?c))]
  :then
  [(:wjl::Hit :loc ?loc)])

(:wat::rete::defquery :wjl::q-Hit
  :params []
  :when [(?fact <- :wjl::Hit)])

(:wat::core::defn :wjl::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wjl::q-Hit))))

(:wat::core::defn :wjl::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :wjl::run [rule <- :wat::rete::Rule] -> :wat::rete::Session
  (:wat::rete::compile-all
    (:wat::core::PersistentVector rule)
    (:wat::core::PersistentVector (:wjl::q-Hit))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [inline (:wjl::run (:wjl::wind-above-temp-inline))
                    where  (:wjl::run (:wjl::wind-above-temp-where))]
    (:wjl::line 1 "empty-inline"
      (:wjl::n-hit (:wat::core::match (:wat::rete::fire-rules inline) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wjl::line 2 "temp-only-inline"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert inline (:wjl::Temp :c 10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wjl::line 3 "wind-only-inline"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert inline (:wjl::Wind :kph 20 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    ;; Clara 0 — wind is not above temp. Wat must print 0 to be in parity.
    (:wjl::line 4 "below-inline"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert inline
            (:wjl::Temp :c 10 :loc "MCI")
            (:wjl::Wind :kph 5 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    ;; Clara 1 — 20 > 10. Wat must print 1 to be in parity.
    (:wjl::line 5 "above-inline"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert inline
            (:wjl::Temp :c 10 :loc "MCI")
            (:wjl::Wind :kph 20 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    ;; Clara 0 — equal is not >.
    (:wjl::line 6 "equal-inline"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert inline
            (:wjl::Temp :c 10 :loc "MCI")
            (:wjl::Wind :kph 10 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    ;; CONTROL: same facts as 4 / 5, predicate in a :where. Both engines.
    (:wjl::line 7 "below-where"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert where
            (:wjl::Temp :c 10 :loc "MCI")
            (:wjl::Wind :kph 5 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wjl::line 8 "above-where"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert where
            (:wjl::Temp :c 10 :loc "MCI")
            (:wjl::Wind :kph 20 :loc "MCI")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    ;; Clara 1 — only MCI is above; ORD is below.
    (:wjl::line 9 "two-locs-inline"
      (:wjl::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert inline
            (:wjl::Temp :c 10 :loc "MCI")
            (:wjl::Wind :kph 20 :loc "MCI")
            (:wjl::Temp :c 10 :loc "ORD")
            (:wjl::Wind :kph 5 :loc "ORD")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
