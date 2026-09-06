;; wat-scripts/perf/grid/retract-multiplicity.wat — CORRECTNESS AXIS: retract of one duplicate.
;;
;; Acceptance test for `factbag::remove-one`. Duplicate ONLY the retracted key:
;;   F(0)×2, F(k)×1 for k>0, G(k)×1 for every k; Out(k) :- F(k) AND G(k);
;;   fire; retract F(0) ONCE; re-fire.
;;
;; Sole witness is key 0. Other keys are unique on both engines, so the justified
;; derived-multiplicity split (Clara bag vs wat set on derived facts) cannot dominate.
;; After remove-one: wat/oracle/Clara all `[0 1 2]` at size [3].
;;
;; ⛔ :derived is the FULL SORTED bag of Out keys — no `set`, no `distinct`. Sort, do not dedup.
;;
;; NO `gen-retract-multiplicity.sh` TWIN, DELIBERATELY. `run-all.sh:81-87` discovers a perf axis
;; as `<axis>.wat` WITH `gen-<axis>.sh`. Static `retract-multiplicity.clj`, same as
;; `parametric-erasure`.
;;
;; Usage (stdin = an i64 vector [items]; stdout = one #grid/Result EDN line):
;;   echo '[3]' | cargo wat ./wat-scripts/perf/grid/retract-multiplicity.wat
;;   => #grid/Result {:axis "retract-multiplicity" :size [3] :derived [0 1 2] :native-ns N ...}

(:wat::core::defrecord :rm::F   [k <- :wat::core::i64])
(:wat::core::defrecord :rm::G   [k <- :wat::core::i64])
(:wat::core::defrecord :rm::Out [k <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :rm::q-Out
  :params []
  :when [(?fact <- :rm::Out)])


;; build-rules — the single join: Out(k) :- F(k) AND G(k). Two equal F(k) × one G(k) fire
;; two activations, so Out carries multiplicity if the query does not collapse it.
(:wat::core::defn :rm::build-rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::rete::Rule :name "out"
      :lhs (:wat::core::PersistentVector
        (:wat::core::quote (:rm::F (?k <- :k)))
        (:wat::core::quote (:rm::G (?k <- :k))))
      :rhs (:wat::core::PersistentVector
        (:wat::core::quote (:rm::Out ?k))))))

;; seed session items — F(i)+G(i) once each for i in [0, items), then ONE extra F(0).
;; Duplicate ONLY the retracted key so the justified derived-multiplicity split cannot dominate.
(:wat::core::defn :rm::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    session
    (:wat::core::PersistentVector/conj
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                        -> (:wat::core::PersistentVector :- [:wat::core::Record])
          (:wat::core::PersistentVector/conj
            (:wat::core::PersistentVector/conj acc (:rm::F i))
            (:rm::G i)))
        (:wat::core::PersistentVector)
        (:wat::core::range 0 items))
      (:rm::F 0))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :rm::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]).
(:wat::core::defn :rm::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived Out fact's key, SORTED, NOT DEDUPED. Two Out(1) stay
;; two 1s. A missing/extra/collapsed key shows up in the byte-for-byte compare.
(:wat::core::defn :rm::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [codes (:wat::core::into (:wat::core::Vector :wat::core::i64)
                            (:wat::core::map
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:rm::Out/k f)))
                              (:wat::rete::query fired (:rm::q-Out))))]
    (:rm::vec->pvec (:wat::core::sort codes))))

(:wat::core::defn :rm::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    items   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [items]")
                    rules   (:rm::build-rules)
                    seeded  (:rm::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:rm::q-Out))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))) items)
                    ;; fire; retract F(0) ONCE; the retracted session is what both engines re-fire.
                    ;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
                    ;; `fire-rules$oracle staged` (the re-fire) and not a first-fire leftover.
                    pre     (:rm::fire seeded)
                    staged  (:wat::rete::retract pre (:rm::F 0))
                    n0      (:wat::time::now)
                    fired   (:rm::fire staged)
                    n1      (:wat::time::now)
                    derived (:rm::derived-vector fired)
                    nat-ns  (:rm::ns-between n0 n1)
                    ;; ORACLE — re-fired on the SAME retracted session. Value semantics make
                    ;; the two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "retract-multiplicity" :size (:wat::core::PersistentVector items) :derived derived :native-ns nat-ns :oracle-derived (:rm::derived-vector ofired) :oracle-ns (:rm::ns-between o0 o1)))))
