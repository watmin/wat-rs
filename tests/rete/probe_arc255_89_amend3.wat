;; Stone 255.89 amend 3 — inline cond and the two oracle reads, both spellings.

(wat.core/defrecord w3/Req [k :- wat.type/i64])
(wat.core/defrecord w3/Hit [k :- wat.type/i64])
(wat.core/defrecord n3/W [k :- wat.type/i64])

(wat.rete/defrule w3k/inline
  :when [(:w3::Req (?k :- :k) (:wat::rete::core::cond ((:wat::rete::i64::> :k 100) true) (:else false)))]
  :then [(:w3::Hit :k ?k)])

(wat.rete/defrule w3s/inline
  :when [(w3/Req (?k :- :k) (wat.rete.core/cond ((wat.rete.i64/> :k 100) true) (:else false)))]
  :then [(w3/Hit :k ?k)])

(wat.rete/defquery w3/q :params [] :when [(:w3::Hit (?k :- :k))])

(wat.core/defn w3/count [ns :- wat.type/keyword] :- wat.type/i64
  (wat.core/let
    [rules (wat.rete/collect-rules ns)
     s0    (wat.core/match (wat.rete/compile-all rules (wat.type/PersistentVector :- [wat.rete/Query] (w3/q)))
              [wat.rete/CompileOutcome.Compiled {:session __session} __session]
              [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type}
                (wat.kernel/assertion-failed! :message "compile")])
     s1    (wat.core/match (wat.rete/insert s0 (w3/Req :k 150) (w3/Req :k 50))
              [wat.rete/InsertOutcome.Inserted {:session __staged} __staged]
              [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count}
                (wat.kernel/assertion-failed! :message "insert")])
     fired (wat.core/match (wat.rete/fire-rules s1)
              [wat.rete/FireOutcome.Fired {:value __fired} __fired]
              [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds}
                (wat.kernel/assertion-failed! :message "fire")]
              [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still}
                (wat.kernel/assertion-failed! :message "cap")])]
    (wat.core/length (wat.rete/query fired (w3/q)))))

(wat.core/defn user/inline-kw [] :- wat.type/i64 (w3/count :w3k))
(wat.core/defn user/inline-sym [] :- wat.type/i64 (w3/count :w3s))

(wat.core/defn w3/neg-len [src :- wat.type/String] :- wat.type/i64
  (wat.core/let [forms (wat.core/match (wat.core/read-string src)
                    [wat.core/ReadOutcome.Forms {:forms __forms} __forms]
                    [wat.core/ReadOutcome.Malformed {:cause __cause}
                      (wat.kernel/assertion-failed! :message "read")])]
    (wat.core/length (wat.rete/rule-negates-in (wat.core/first forms)))))

(wat.core/defn user/neg-kw [] :- wat.type/i64
  (w3/neg-len "(:wat::rete::or (:n3::W) (:wat::rete::not (:n3::W)))"))

(wat.core/defn user/neg-sym [] :- wat.type/i64
  (w3/neg-len "(wat.rete/or (n3/W) (wat.rete/not (n3/W)))"))
