;; wat-scripts/perf/grid/retract-accum-derived.wat — COMPOUND CORRECTNESS AXIS: cell
;; (record, present, derived, nonleading, na) of the peragrare grid — cell #5 of
;; docs/arc/2026/06/278-rules-engine/strike-grid-remaining-compound-cells/DESIGN.md.
;;
;; A duplicate-retract (retract-multiplicity's `R` mechanism — remove-one, not remove-all)
;; applied to the accumulate's OWN `:from` SOURCE (accum-over-derived's `A` mechanism — a
;; type the SAME ruleset also derives, needing cross-round supersession). THIS AXIS IS THEIR
;; INTERSECTION on the SAME field — read both first:
;;
;;   retract-multiplicity  (record, present, none,    none,       na) — F(0)×2 feeds a
;;                         plain join; F is never derived by any rule in that ruleset.
;;   accum-over-derived    (record, absent,  derived, nonleading, na) — Seed anchors an
;;                         accumulate over Step, which the SAME ruleset derives via
;;                         Step(k):-Step(k-1); Step is never duplicated or retracted.
;;   THIS AXIS              (record, present, derived, nonleading, na) — accum-over-derived's
;;                         EXACT shape, but Step(0) — the accumulate's own `:from` source,
;;                         AND the seed the cascade derives everything else from — is
;;                         duplicated (retract-multiplicity's technique), then ONE copy is
;;                         retracted and the session re-fired.
;;
;; Seed;  Step(0)×2 (duplicated);  Step(k) :- Step(k-1) for k in [1,depth];
;; Tally(n) :- Seed AND [?n <- (acc::count) :from Step];
;; fire;  retract Step(0) ONCE (remove-one);  re-fire.
;;
;; WHY THIS IS THE RISK, not a formality: retract-multiplicity's remove-one cure has only
;; ever been driven against a type NO rule in the same ruleset also derives — the retracted
;; fact's identity is unambiguous, nothing else claims to produce it. Here Step(0) is BOTH a
;; directly-seeded (base) fact we duplicate-then-retract AND the root of a derivation
;; cascade that the SAME accumulate (accum-over-derived's own cure: evaluate Tally once,
;; AFTER Step closes) depends on. If remove-one's bookkeeping does not correctly narrow to
;; the ONE retracted token when that token's type is ALSO a derivation root, or if
;; retracting one of two duplicate Step(0) tokens confuses which downstream Step(1..depth)
;; derivations remain justified, this is where that would show: either a leaked intermediate
;; Tally (accum-over-derived's own failure shape, now triggered by a RETRACT instead of a
;; fixpoint round), a stale Step(k) surviving the retract when its sole justification should
;; have followed Step(0) down, or the duplicate surviving remove-one entirely (retract-
;; multiplicity's own failure shape, remove-ALL when remove-ONE was asked).
;;
;; ⛔ ONE CORRECTNESS SIZE, not a sweep. No `gen-retract-accum-derived.sh` — a generator
;; would drag a correctness proof onto the perf ladder (`run-all.sh:81-87`,
;; `check-grid-speed.sh` treats `:accuracy :MISMATCH` as a gate failure). Static
;; `retract-accum-derived.clj`.
;;
;; Depth is a stated choice: 9 — same as accum-over-derived/accum-lead-derived, for the same
;; reason (past the probe's two intermediate states, 4.5x). If the oracle does not
;; terminate, hits the round cap, or takes minutes here, that is STOP-3 — do not shrink
;; until it is quiet.
;;
;; :derived = sorted, NOT deduped: enc(0,k,0) per derived Step level k in [1,depth] plus
;; enc(1,0,n) per Tally. PREDICTED (under correct truth-maintenance/remove-one semantics,
;; re-derived from the formula, NOT read off a run): remove-one on Step(0) returns the
;; population to exactly accum-over-derived's own canonical state — one Step(0), one
;; Step(k) per k in [1,depth] — so Tally n = depth + 1 and expected count = depth + 1 = 10,
;; same as accum-over-derived's own number. ⛔ THIS PREDICTION IS THE HYPOTHESIS UNDER TEST,
;; not a settled fact: whether the duplicate's presence during the FIRST fire (before the
;; retract) causes any of Step(1..depth) to be over-derived and left un-superseded by the
;; retract is exactly the compound risk this cell exists to measure — see the PIN.
;;
;; Variable MUST be named `staged` so GRID_SKIP_ORACLE / axes_live rewrite
;; `fire-rules$oracle staged`.
;;
;; Usage (stdin = an i64 vector [depth]; stdout = one #grid/Result EDN line):
;;   echo '[9]' | cargo wat ./wat-scripts/perf/grid/retract-accum-derived.wat

(:wat::core::defrecord :rad::Seed  [id <- :wat::core::i64])
(:wat::core::defrecord :rad::Step  [level <- :wat::core::i64])
(:wat::core::defrecord :rad::Tally [n <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :rad::q-Step
  :params []
  :when [(?fact <- :rad::Step)])


(:wat::rete::defquery :rad::q-Tally
  :params []
  :when [(?fact <- :rad::Tally)])


;; build-step k — Step(k) :- Step(k-1). Same as accum-over-derived's per-level rule.
(:wat::core::defn :rad::build-step [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c (:wat::core::quasiquote (:rad::Step (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))
                    t (:wat::core::quasiquote (:rad::Step (:wat::core::unquote k)))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c)
      :rhs (:wat::core::PersistentVector t))))

;; tally — Seed AND count of every Step. Seed anchors the join (nonleading), exactly
;; accum-over-derived's own tally-rule, unchanged.
(:wat::core::defn :rad::tally-rule [] -> :wat::rete::Rule
  (:wat::rete::Rule :name "tally"
    :lhs (:wat::core::PersistentVector
      (:wat::core::quote (:rad::Seed (?id <- :id)))
      (:wat::core::quote (?n <- (:wat::rete::acc::count) :from (:rad::Step))))
    :rhs (:wat::core::PersistentVector
      (:wat::core::quote (:rad::Tally ?n)))))

(:wat::core::defn :rad::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:rad::build-step k)))
    (:wat::core::PersistentVector (:rad::tally-rule))
    (:wat::core::range 1 (:wat::core::i64::+ depth 1))))

;; Empty (PersistentVector :- [Record]) so Seed and Step can share one batch. A two-element
;; literal infers from the first element and refuses the second (homogeneous PV; same
;; check-time refusal insert-all has for mixed Records) — accum-over-derived's own note.
(:wat::core::defn :rad::empty-records [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector))

;; seed-facts — Seed(0), Step(0) TWICE (the duplicate — the accumulate's own source,
;; ALSO the cascade's root), exactly retract-multiplicity's "duplicate ONLY the retracted
;; key" care.
(:wat::core::defn :rad::seed-facts [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector/conj
    (:wat::core::PersistentVector/conj
      (:wat::core::PersistentVector/conj
        (:rad::empty-records)
        (:rad::Seed :id 0))
      (:rad::Step :level 0))
    (:rad::Step :level 0)))

(:wat::core::defn :rad::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all session (:rad::seed-facts))
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
     (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :rad::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
     (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
     (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :rad::enc [kind <- :wat::core::i64  level <- :wat::core::i64  id <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+
    (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) (:wat::core::i64::* level 1000000000))
    id))

(:wat::core::defn :rad::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector — sorted, NOT deduped. Derived Step levels (level > 0) plus every Tally.
(:wat::core::defn :rad::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:rad::enc 0 (:rad::Step/level f) 0)))
            (:wat::core::filter
              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool
                (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                  (:wat::core::i64::> (:rad::Step/level f) 0)))
              (:wat::rete::query fired (:rad::q-Step)))))
     c1 (:wat::core::into c0
          (:wat::core::map
            (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
              (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")]
                (:rad::enc 1 0 (:rad::Tally/n f))))
            (:wat::rete::query fired (:rad::q-Tally))))]
    (:rad::vec->pvec (:wat::core::sort c1))))

(:wat::core::defn :rad::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    depth   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [depth]")
                    rules   (:rad::build-rules depth)
                    seeded  (:rad::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:rad::q-Step) (:rad::q-Tally))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))
                    ;; fire; retract Step(0) ONCE; the retracted session is what both engines
                    ;; re-fire. Variable MUST be named `staged` so GRID_SKIP_ORACLE /
                    ;; axes_live rewrite `fire-rules$oracle staged` (the re-fire).
                    pre     (:rad::fire seeded)
                    staged  (:wat::rete::retract pre (:rad::Step 0))
                    n0      (:wat::time::now)
                    fired   (:rad::fire staged)
                    n1      (:wat::time::now)
                    derived (:rad::derived-vector fired)
                    nat-ns  (:rad::ns-between n0 n1)
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "retract-accum-derived" :size (:wat::core::PersistentVector depth) :derived derived :native-ns nat-ns :oracle-derived (:rad::derived-vector ofired) :oracle-ns (:rad::ns-between o0 o1)))))
