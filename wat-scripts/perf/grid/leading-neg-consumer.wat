;; wat-scripts/perf/grid/leading-neg-consumer.wat — COMPOUND CORRECTNESS AXIS: cell
;; (record, absent, none, leading, consumed) of the peragrare grid — cell #4 of
;; docs/arc/2026/06/278-rules-engine/strike-grid-remaining-compound-cells/DESIGN.md.
;;
;; A positive consumer (neg-consumer's `N` mechanism — a POSITIVE rule consuming a fact
;; gated by a negation elsewhere) sitting downstream of a LEADING gate (leading-exists's `L`
;; mechanism — a parentless `:exists`, re-evaluated per fixpoint round, forced through
;; multiple rounds by an inert cascade). THIS AXIS IS THEIR INTERSECTION — read both first:
;;
;;   neg-consumer     (record, absent, none, none,    consumed) — Ok :- Item, NOT Bad is
;;                    the gate; Item is a PARENT condition, so the negation is NOT leading.
;;                    Final :- Ok, Tag is the positive consumer. ONE fixpoint round (no
;;                    forcing cascade) — the round-count-independence property is untested.
;;   leading-exists   (record, absent, none, leading, na)       — a LEADING :exists over
;;                    Wind, observed through a QUERY, across 6 forced fixpoint rounds
;;                    (inert S1..S6 cascade). Nothing CONSUMES the exists's output — it is
;;                    read directly by the query, never joined into a further rule.
;;   THIS AXIS        (record, absent, none, leading, consumed) — leading-exists's Wind +
;;                    inert-cascade shape, but the leading `:exists` now feeds a Signal
;;                    fact that TWO further rules consume positively: Ok :- Signal, NOT Bad
;;                    (Bad never seeded, vacuous) and Final :- Ok, Tag — neg-consumer's own
;;                    two-stage positive-consumer chain, downstream of the leading gate
;;                    instead of a parent-anchored one.
;;
;; Wind(loc)×2 per loc in [0,items);  Tag(loc) per loc in [0,items);  Bad — NEVER seeded;
;; S1(1) seeded, S2..S6 derived — INERT cascade forcing 6 fixpoint rounds, touches nothing
;; Wind/Signal/Ok/Final-related;
;;   Signal(loc) :- (exists Wind(loc))                LEADING — no parent, no anchor
;;   Ok(loc)     :- Signal(loc), NOT Bad(loc)          consumes the leading gate positively
;;   Final(loc)  :- Ok(loc), Tag(loc)                  the POSITIVE CONSUMER two hops downstream
;;
;; WHY THIS IS THE RISK, not a formality: leading-exists's cure stops a leading non-monotonic
;; condition from re-emitting one token per fixpoint round, PROVEN where nothing consumes
;; that token beyond a direct query. neg-consumer's cure makes positive dependencies
;; propagate a rule's stratum correctly, PROVEN where the gate it sits downstream of has a
;; PARENT condition anchoring it. Neither fixture drives a POSITIVE rule chain consuming the
;; output of a LEADING gate specifically. If leading-exists's per-round re-emission defect
;; is not fully closed at the SOURCE (Signal) but only masked at the query layer that found
;; it, this is where that would show: Final duplicated `rounds`-many times per key, or worse,
;; Ok/Final's stratum placement misjudging a leading (parentless) upstream producer and
;; either dropping Final entirely (neg-consumer's pre-cure failure shape) or leaking stale
;; copies across rounds (leading-exists's pre-cure failure shape) — propagated one and two
;; hops further downstream than either parent ever drove it.
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep. No `gen-leading-neg-consumer.sh` — a generator would
;; drag a correctness proof onto the perf ladder (`run-all.sh:81-87`, `check-grid-speed.sh`
;; treats `:accuracy :MISMATCH` as a gate failure). Static `leading-neg-consumer.clj`.
;;
;; Items is a stated choice: 20 — same as leading-exists's own correctness size, for the
;; same reason (small, already exercises the distinct-inner-binding rule: two Winds at one
;; loc => ONE {?loc} token).
;;
;; :derived = sorted, NOT deduped: the Final locs. Correct answer: exactly [0..items), one
;; per loc — Bad never fires, Tag is seeded for every loc, and Signal binds each distinct
;; loc exactly once (Clara `test-simple-exists` semantics, the same contract leading-exists
;; verifies). Under EITHER parent's pre-cure defect this count would move: `items * rounds`
;; if the leading re-emission resurfaces, or `0` if positive-stratum propagation fails
;; through a leading (parentless) producer.
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[20]' | cargo wat ./wat-scripts/perf/grid/leading-neg-consumer.wat

(:wat::core::defrecord :lnc::Wind   [loc <- :wat::core::i64])
(:wat::core::defrecord :lnc::Bad    [loc <- :wat::core::i64])
(:wat::core::defrecord :lnc::Tag    [loc <- :wat::core::i64])
(:wat::core::defrecord :lnc::Signal [loc <- :wat::core::i64])
(:wat::core::defrecord :lnc::Ok     [loc <- :wat::core::i64])
(:wat::core::defrecord :lnc::Final  [loc <- :wat::core::i64])
(:wat::core::defrecord :lnc::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :lnc::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :lnc::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :lnc::S4 [k <- :wat::core::i64])
(:wat::core::defrecord :lnc::S5 [k <- :wat::core::i64])
(:wat::core::defrecord :lnc::S6 [k <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :lnc::q-Final
  :params []
  :when [(?fact <- :lnc::Final)])


;; THE WITNESS CHAIN: a LEADING :exists producing Signal, consumed positively by Ok
;; (NOT Bad, Bad never fires), consumed positively again by Final (joined with Tag).
;; The inert S1..S6 cascade forces six fixpoint rounds, exactly leading-exists's own —
;; nothing here mentions Wind/Signal/Ok/Final, that is the point.
(:wat::core::defn :lnc::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::rete::Rule :name "signal"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quote (:wat::rete::exists (:lnc::Wind (?loc <- :loc)))))
      :rhs (:wat::core::PersistentVector
        (:wat::core::quote (:lnc::Signal ?loc))))
    (:wat::rete::Rule :name "ok"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quote (:lnc::Signal (?loc <- :loc)))
        (:wat::core::quote (:wat::rete::not (:lnc::Bad (?loc <- :loc)))))
      :rhs (:wat::core::PersistentVector
        (:wat::core::quote (:lnc::Ok ?loc))))
    (:wat::rete::Rule :name "final"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quote (:lnc::Ok  (?loc <- :loc)))
        (:wat::core::quote (:lnc::Tag (?loc <- :loc))))
      :rhs (:wat::core::PersistentVector
        (:wat::core::quote (:lnc::Final ?loc))))
    (:wat::rete::Rule :name "r2"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S1 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S2 ?k))))
    (:wat::rete::Rule :name "r3"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S2 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S3 ?k))))
    (:wat::rete::Rule :name "r4"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S3 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S4 ?k))))
    (:wat::rete::Rule :name "r5"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S4 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S5 ?k))))
    (:wat::rete::Rule :name "r6"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S5 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:lnc::S6 ?k))))))

;; Seed: Wind(i) TWICE for each i in [0,items), Tag(i) once for each i, plus one S1 to
;; start the cascade. Bad is NEVER seeded.
(:wat::core::defn :lnc::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    session
    (:wat::core::PersistentVector/conj
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                        -> (:wat::core::PersistentVector :- [:wat::core::Record])
          (:wat::core::let [a2 (:wat::core::PersistentVector/conj acc (:lnc::Wind i))
                            a3 (:wat::core::PersistentVector/conj a2 (:lnc::Wind i))]
            (:wat::core::PersistentVector/conj a3 (:lnc::Tag i))))
        (:wat::core::PersistentVector)
        (:wat::core::range 0 items))
      (:lnc::S1 1))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :lnc::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :lnc::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; THE ACCURACY WITNESS. Sorted Final locs, NOT deduped. Under a leading re-emission leak
;; this vector is `rounds` times too long; under a stratum-propagation failure it is empty.
(:wat::core::defn :lnc::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :wat::core::i64)
                            (:wat::core::map
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
                                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                                  (:lnc::Final/loc f)))
                              (:wat::rete::query fired (:lnc::q-Final))))]
    (:lnc::vec->pvec (:wat::core::sort codes))))

(:wat::core::defn :lnc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:lnc::build-rules)
                    staged  (:lnc::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:lnc::q-Final))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))) items)
                    n0      (:wat::time::now)
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    n1      (:wat::time::now)
                    derived (:lnc::derived-vector fired)
                    nat-ns  (:lnc::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "leading-neg-consumer" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns :oracle-derived (:lnc::derived-vector ofired) :oracle-ns (:lnc::ns-between o0 o1)))))
