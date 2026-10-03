;; wat-scripts/perf/grid/where-test-chain.wat — Test→Join→Test→Test (Clara test-simple-test).
;;
;; Twin of where-test-chain.clj. Spoken order used to derive nothing on native
;; (3.7 walked Test children of a HashJoin, not the next Test). Joins-first was
;; already green. 1↔2 must print the same n= / pair. Clara 0.24.0 does.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-test-chain.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-test-chain.clj

(wat.core/defn wtc/row-count [] :- wat.type/i64 2)

(wat.core/defrecord wtc/Temp [c :- wat.type/i64 loc :- wat.type/String])
(wat.core/defrecord wtc/Pair [a :- wat.type/i64 b :- wat.type/i64])

;; ROW 1 — spoken: filter, join, filter, filter. t1<20, t2<20, t1<t2.
;; Facts 15/10/80 at MCI → one pair {10 15}.
(wat.rete/defrule wtc/spoken
  :when
  [(wtc/Temp (?t1 :- :c) (?loc :- :loc) (wat.rete.i64/< ?t1 20))
   (wtc/Temp (?t2 :- :c) (?loc :- :loc) (wat.rete.i64/< ?t2 20))
   (wat.rete/where (wat.rete.i64/< ?t1 ?t2))]
  :then
  [(wtc/Pair :a ?t1 :b ?t2)])

;; ROW 2 — joins first, then the three filters. Same set as row 1.
(wat.rete/defrule wtc/join-first
  :when
  [(wtc/Temp (?t1 :- :c) (?loc :- :loc) (wat.rete.i64/< ?t1 20))
   (wtc/Temp (?t2 :- :c) (?loc :- :loc) (wat.rete.i64/< ?t2 20))
   (wat.rete/where (wat.rete.i64/< ?t1 ?t2))]
  :then
  [(wtc/Pair :a ?t1 :b ?t2)])

(wat.rete/defquery wtc/q-Pair
  :params []
  :when [(wtc/Pair (?a :- :a) (?b :- :b))])


(wat.core/defn wtc/build-rules [row :- wat.type/i64] :- (wat.type/PersistentVector :- [wat.rete/Rule])
  (wat.type/PersistentVector :- [wat.rete/Rule]
    (wat.core/cond
      ((wat.core/= row 1) (wtc/spoken))
      ((wat.core/= row 2) (wtc/join-first))
      (:else
        (wat.kernel/assertion-failed! :message (wat.string/concat "where-test-chain: unknown row " (wat.i64/to-string row)))))))

(wat.core/defn wtc/seed [session :- wat.rete/Session] :- wat.rete/Session
  (wat.core/match (wat.rete/insert session
    (wtc/Temp :c 15 :loc "MCI")
    (wtc/Temp :c 10 :loc "MCI")
    (wtc/Temp :c 80 :loc "MCI")) [wat.rete/InsertOutcome.Inserted {:session __staged} __staged] [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (wat.kernel/assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))

(wat.core/defn wtc/render [fired :- wat.rete/Session] :- wat.type/String
  (wat.core/let [pairs (wat.rete/query fired (wtc/q-Pair))
                    n     (wat.core/length pairs)
                    shown (wat.core/foldl
                             (wat.core/fn [acc :- wat.type/String  p :- wat.type/PersistentMap]
                               :- wat.type/String
                               (wat.string/concat acc
                                 (wat.string/concat " "
                                   (wat.string/concat
                                     (wat.i64/to-string
                                       (wat.core.Option/expect
                                         (wat.core/get p "?a")
                                         "q-Pair: ?a"))
                                     (wat.string/concat ","
                                       (wat.i64/to-string
                                         (wat.core.Option/expect
                                           (wat.core/get p "?b")
                                           "q-Pair: ?b")))))))
                             ""
                             pairs)]
    (wat.string/concat
      (wat.string/concat " n=" (wat.i64/to-string n))
      (wat.string/concat " ->" shown))))

(wat.core/defn wtc/run-row [row :- wat.type/i64] :- wat.type/String
  (wat.core/let [rules (wtc/build-rules row)
                    rule  (wat.core/first rules)
                    fired (wat.core/match (wat.rete/fire-rules (wtc/seed (wat.core/match (wat.rete/compile-all rules (wat.type/PersistentVector :- [wat.rete/Query] (wtc/q-Pair))) [wat.rete/CompileOutcome.Compiled {:session __session} __session] [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (wat.kernel/assertion-failed! :message "compile: the rule set may not terminate")]))) [wat.rete/FireOutcome.Fired {:value __fired} __fired] [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (wat.kernel/assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (wat.kernel/assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
                    name  (wat.core/foldl
                             (wat.core/fn [acc :- wat.type/String  seg :- wat.type/String]
                               :- wat.type/String seg)
                             (wat.rete.Rule/name rule)
                             (wat.string/split (wat.rete.Rule/name rule) "::"))]
    (wat.string/concat
      (wat.string/concat "row " (wat.i64/to-string row))
      (wat.string/concat (wat.string/concat " " name) (wtc/render fired)))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/nil  row :- wat.type/i64] :- wat.type/nil
      (wat.kernel/println (wtc/run-row row)))
    nil
    (wat.core/range 1 3)))
