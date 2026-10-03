;; RELAND 10: drive fire-rules DIRECTLY on the same rules and facts the
;; generated sift-rules body uses — outside the service, so a raise is the
;; service's own error instead of the client's Lost/"disconnected".
(wat.core/defrecord usr/Temp [c :- wat.type/i64])
(wat.core/defrecord usr/Hot  [c :- wat.type/i64])
(wat.core/defrecord usr/Warn [c :- wat.type/i64])

(wat.core/defn user/hot-q [] :- wat.rete/Query
  (wat.rete/make-query "usr::Hot" (wat.core/quote [])
    (wat.core/quote [(?fact :- usr/Hot)])))

(wat.core/defn user/warn-q [] :- wat.rete/Query
  (wat.rete/make-query "usr::Warn" (wat.core/quote [])
    (wat.core/quote [(?fact :- usr/Warn)])))

(wat.core/defn user/run-one [session :- wat.rete/Session encoded :- wat.type/String]
  :- (wat.type/PersistentVector :- [wat.type/Value])
  (wat.core/let
    [fired (wat.core/match (wat.rete/fire-rules
             (wat.core/match (wat.rete/insert session (wat.edn/read encoded)) [wat.rete/InsertOutcome.Inserted {:session __staged} __staged] [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (wat.kernel/assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [wat.rete/FireOutcome.Fired {:value __fired} __fired] [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (wat.kernel/assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (wat.kernel/assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
    (wat.core/concat
      (wat.core/into (wat.type/PersistentVector :- [wat.type/Value])
        (wat.core/map
          (wat.core/fn [p :- wat.type/PersistentMap] :- wat.type/Value
            (wat.core.Option/expect (wat.core/get p "?fact") "sift-rules: ?fact"))
          (wat.rete/query fired (user/hot-q))))
      (wat.core/into (wat.type/PersistentVector :- [wat.type/Value])
        (wat.core/map
          (wat.core/fn [p :- wat.type/PersistentMap] :- wat.type/Value
            (wat.core.Option/expect (wat.core/get p "?fact") "sift-rules: ?fact"))
          (wat.rete/query fired (user/warn-q)))))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [rules (wat.type/PersistentVector :- [wat.rete/Rule]
             (wat.rete/make-rule "usr::hot-rule"
               (wat.core/quote [(usr/Temp (?c :- :c) (wat.rete.i64/> ?c 50))])
               (wat.core/quote [(usr/Hot :c ?c)]))
             (wat.rete/make-rule "usr::warn-rule"
               (wat.core/quote [(usr/Temp (?c :- :c) (wat.rete.i64/> ?c 50))])
               (wat.core/quote [(usr/Warn :c ?c)])))
     queries (wat.type/PersistentVector :- [wat.rete/Query]
               (user/hot-q)
               (user/warn-q))
     session (wat.core/match (wat.rete/compile-all rules queries) [wat.rete/CompileOutcome.Compiled {:session __session} __session] [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (wat.kernel/assertion-failed! :message "compile: the rule set may not terminate")])
     one (wat.edn/write (usr/Temp :c 60))
     items1 (user/run-one session one)]
    (wat.kernel/println "ONE-FACT")
    (wat.kernel/pprintln (wat.core/count items1))
    (wat.core/let
      [idxs (wat.core/range 0 240)
       items240 (wat.core/foldl
                  (wat.core/fn [acc :- (wat.type/PersistentVector :- [wat.type/Value])
                                   i :- wat.type/i64]
                    :- (wat.type/PersistentVector :- [wat.type/Value])
                    (wat.core/let
                      [hot? (wat.i64/< i 30)
                       c    (wat.core/if hot? 60 10)
                       enc  (wat.edn/write (usr/Temp :c c))]
                      (wat.core/concat acc (user/run-one session enc))))
                  (wat.type/PersistentVector :- [wat.type/Value])
                  idxs)]
      (wat.kernel/println "TWO-FORTY")
      (wat.kernel/pprintln (wat.core/count items240))
      nil)))
