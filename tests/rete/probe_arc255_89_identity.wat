;; Stone 255.89 amend 2 — the same rule or form in each spelling, the same value.
;; Keyword payloads of a keyword program stay the written keyword. A reference
;; symbol is the other spelling of that identity. Field keywords (:k, :n, :count)
;; stay value keywords.

(wat.core/defrecord id89/W [k :- wat.type/i64])
(wat.core/defrecord id89/Hit [n :- wat.type/i64])
(wat.core/defrecord id89/Anchor [x :- wat.type/i64])
(wat.core/defrecord id89/Rate [count :- wat.type/i64 window :- wat.type/i64])
(wat.core/defstruct id89/Paper [outcome :- wat.type/String])

(wat.rete/defrule id89kw/via-exists
  :when [(:wat::rete::exists (:id89::W (?w :- :k)))]
  :then [(:id89::Hit :n 1)])

(wat.rete/defrule id89sym/via-exists
  :when [(wat.rete/exists (id89/W (?w :- :k)))]
  :then [(:id89::Hit :n 1)])

(wat.rete.core/defn id89/make-rate
  [c :- wat.type/i64
   w :- wat.type/i64]
  :- id89/Rate
  (id89/Rate :count c :window w))

(wat.rete/defrule id89kw/rate
  :when [(:id89::Anchor (?x :- :x))]
  :then [(:id89::make-rate 7 9)])

(wat.rete/defrule id89sym/rate
  :when [(id89/Anchor (?x :- :x))]
  :then [(id89/make-rate 7 9)])

(wat.rete/defquery id89/q-Hit
  :params []
  :when [(:id89::Hit (?n :- :n))])

(wat.rete/defquery id89/q-Rate
  :params []
  :when [(:id89::Rate (?count :- :count))])

(wat.core/defn user/exists-kw [] :- wat.type/i64
  (wat.core/let
    [rules (wat.rete/collect-rules :id89kw)
     s0    (wat.core/match (wat.rete/compile-all rules (wat.type/PersistentVector :- [wat.rete/Query] (id89/q-Hit))) [wat.rete/CompileOutcome.Compiled {:session __session} __session] [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (wat.kernel/assertion-failed! :message "compile: the rule set may not terminate")])
     s1    (wat.core/match (wat.rete/insert s0 (id89/W :k 1)) [wat.rete/InsertOutcome.Inserted {:session __staged} __staged] [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (wat.kernel/assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (wat.core/match (wat.rete/fire-rules s1) [wat.rete/FireOutcome.Fired {:value __fired} __fired] [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (wat.kernel/assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (wat.kernel/assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     hits  (wat.rete/query fired (id89/q-Hit))]
    (wat.core.Option/expect
      (wat.core/get (wat.core/first hits) "?n")
      "exists-kw: ?n")))

(wat.core/defn user/exists-sym [] :- wat.type/i64
  (wat.core/let
    [rules (wat.rete/collect-rules :id89sym)
     s0    (wat.core/match (wat.rete/compile-all rules (wat.type/PersistentVector :- [wat.rete/Query] (id89/q-Hit))) [wat.rete/CompileOutcome.Compiled {:session __session} __session] [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (wat.kernel/assertion-failed! :message "compile: the rule set may not terminate")])
     s1    (wat.core/match (wat.rete/insert s0 (id89/W :k 1)) [wat.rete/InsertOutcome.Inserted {:session __staged} __staged] [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (wat.kernel/assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (wat.core/match (wat.rete/fire-rules s1) [wat.rete/FireOutcome.Fired {:value __fired} __fired] [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (wat.kernel/assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (wat.kernel/assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     hits  (wat.rete/query fired (id89/q-Hit))]
    (wat.core.Option/expect
      (wat.core/get (wat.core/first hits) "?n")
      "exists-sym: ?n")))

(wat.core/defn user/rate-kw [] :- wat.type/i64
  (wat.core/let
    [rules (wat.rete/collect-rules :id89kw)
     s0    (wat.core/match (wat.rete/compile-all rules (wat.type/PersistentVector :- [wat.rete/Query] (id89/q-Rate))) [wat.rete/CompileOutcome.Compiled {:session __session} __session] [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (wat.kernel/assertion-failed! :message "compile: the rule set may not terminate")])
     s1    (wat.core/match (wat.rete/insert s0 (id89/Anchor :x 0)) [wat.rete/InsertOutcome.Inserted {:session __staged} __staged] [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (wat.kernel/assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (wat.core/match (wat.rete/fire-rules s1) [wat.rete/FireOutcome.Fired {:value __fired} __fired] [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (wat.kernel/assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (wat.kernel/assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     hits  (wat.rete/query fired (id89/q-Rate))]
    (wat.core.Option/expect
      (wat.core/get (wat.core/first hits) "?count")
      "rate-kw: ?count")))

(wat.core/defn user/rate-sym [] :- wat.type/i64
  (wat.core/let
    [rules (wat.rete/collect-rules :id89sym)
     s0    (wat.core/match (wat.rete/compile-all rules (wat.type/PersistentVector :- [wat.rete/Query] (id89/q-Rate))) [wat.rete/CompileOutcome.Compiled {:session __session} __session] [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (wat.kernel/assertion-failed! :message "compile: the rule set may not terminate")])
     s1    (wat.core/match (wat.rete/insert s0 (id89/Anchor :x 0)) [wat.rete/InsertOutcome.Inserted {:session __staged} __staged] [wat.rete/InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (wat.kernel/assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (wat.core/match (wat.rete/fire-rules s1) [wat.rete/FireOutcome.Fired {:value __fired} __fired] [wat.rete/FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (wat.kernel/assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [wat.rete/FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (wat.kernel/assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     hits  (wat.rete/query fired (id89/q-Rate))]
    (wat.core.Option/expect
      (wat.core/get (wat.core/first hits) "?count")
      "rate-sym: ?count")))

(wat.core/defn user/pure-cond-kw [] :- wat.type/bool
  (wat.rete/pure? (wat.core/quote (:wat::core::cond ((:wat::core::> 5 3) 1) (true 0)))))

(wat.core/defn user/pure-cond-sym [] :- wat.type/bool
  (wat.rete/pure? (wat.core/quote (wat.core/cond ((wat.core/> 5 3) 1) (true 0)))))

(wat.core/defn user/admit-kw [] :- wat.type/bool
  (wat.rete/vocabulary-admitted? (wat.core/quote :wat::rete::core::cond)))

(wat.core/defn user/admit-sym [] :- wat.type/bool
  (wat.rete/vocabulary-admitted? (wat.core/quote wat.rete.core/cond)))

(wat.core/defn user/refuse-kw [] :- wat.type/bool
  (wat.rete/vocabulary-admitted? (wat.core/quote :wat::i64::+)))

(wat.core/defn user/refuse-sym [] :- wat.type/bool
  (wat.rete/vocabulary-admitted? (wat.core/quote wat.i64/+)))

(wat.core/defn user/matches-kw [] :- wat.type/bool
  (wat.core/let [p (id89/Paper :outcome "Grace")]
    (wat.form/matches? p
      (:id89::Paper
        (= ?outcome :outcome)
        (= ?outcome "Grace")))))

(wat.core/defn user/matches-sym [] :- wat.type/bool
  (wat.core/let [p (id89/Paper :outcome "Grace")]
    (wat.form/matches? p
      (id89/Paper
        (= ?outcome :outcome)
        (= ?outcome "Grace")))))

(wat.core/defn user/nested-kw [] :- wat.type/i64
  (wat.core/let [mm (wat.core/Option.Some {:value (wat.core/Option.Some {:value 42})})]
    (wat.core/match mm
      [:wat::core::Option.Some {:value [:wat::core::Option.Some {:value x}]} x]
      [:wat::core::Option.Some {:value :wat::core::Option.None} -1]
      [:wat::core::Option.None {} -2]
      [_ -3])))

(wat.core/defn user/nested-sym [] :- wat.type/i64
  (wat.core/let [mm (wat.core/Option.Some {:value (wat.core/Option.Some {:value 42})})]
    (wat.core/match mm
      [wat.core/Option.Some {:value [wat.core/Option.Some {:value x}]} x]
      [wat.core/Option.Some {:value wat.core/Option.None} -1]
      [wat.core/Option.None {} -2]
      [_ -3])))

(wat.core/defn user/total-length-kw [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (:wat::core::length []))))

(wat.core/defn user/total-length-sym [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (wat.core/length []))))

(wat.core/defn user/total-lt-kw [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (:wat::i64::< 1 2))))

(wat.core/defn user/total-lt-sym [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (wat.i64/< 1 2))))

(wat.core/defn user/total-plus-kw [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (:wat::i64::+ 1 2))))

(wat.core/defn user/total-plus-sym [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (wat.i64/+ 1 2))))

(wat.core/defn user/total-subs-kw [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (:wat::string::subs "ab" 0 1))))

(wat.core/defn user/total-subs-sym [] :- wat.type/bool
  (wat.rete/total? (wat.core/quote (wat.string/subs "ab" 0 1))))
