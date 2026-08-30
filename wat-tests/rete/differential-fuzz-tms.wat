;; wat-tests/rete/differential-fuzz-tms.wat — the OPERATION-SEQUENCE fuzzer (truth maintenance).
;;
;; THE PROPERTY, and it is stronger than its two siblings'. Both of those ask "do the two engines
;; agree". This one also asks **PATH INDEPENDENCE**:
;;
;;     running a PROGRAM of inserts, retracts and fires, then firing,
;;         must equal
;;     firing ONCE over the fact multiset that program ends with.
;;
;; Four numbers per case — native-interleaved, oracle-interleaved, native-one-shot,
;; oracle-one-shot — and all four must be equal. Engine disagreement and path dependence are then
;; different failures with the same gate, and the case coordinate says which.
;;
;; WHY PATH INDEPENDENCE IS THE RIGHT PROPERTY HERE. `fire-rules` is contracted as a function of
;; `Session/facts` (it resets `facts = input` and recomputes; `wat/rete/oracle/fire.wat`), so the
;; ONLY way an interleaved run can differ from a one-shot is state surviving a fire that should
;; not. That is families A and C one level up: those were memories accumulating across ROUNDS
;; within a fire; this asks whether anything accumulates across FIRES. Nothing in either sibling
;; fuzzer fires more than twice, and neither ever interleaves an INSERT between fires.
;;
;; WHY NOT "RETRACT MID-FIXPOINT". There is no such hook and there should not be: `retract` is
;; STAGE-ONLY — it removes from `Session/facts` by value and the caller re-fires
;; (`wat/rete/oracle/insert.wat`). So the honest unit of interleaving is the OPERATION, not the
;; round, and that is what this file generates.
;;
;; THE MODEL IS A MULTISET, NOT A SET, and that is load-bearing. `insert` appends and may
;; duplicate; `retract` removes EVERY fact equal to its argument. So a program that inserts A0
;; twice leaves two facts and a class-scan query returns two rows — a set model would predict one
;; and the gate would fail on the MODEL rather than the engine. `final-facts` below replays the
;; program with exactly those semantics.

(:wat::core::defrecord :wat-tests::rete::tms::A [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::tms::B [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::tms::C [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::tms::D [k <- :wat::core::i64])

;; C is DERIVED from A; D consumes the derived C and is gated by a negation over an inserted B.
;; So a retraction of A must un-derive C transitively into D, and an insertion of B must kill D
;; without touching C — two different un-derivation paths from one program.
(:wat::core::defn :wat-tests::rete::tms::rules []
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::rete::Rule :name "r1"
      :lhs (:wat::core::PersistentVector
             (:wat::core::quasiquote (:wat-tests::rete::tms::A (?k <- :k))))
      :rhs (:wat::core::PersistentVector
             (:wat::core::quasiquote (:wat-tests::rete::tms::C ?k))))
    (:wat::rete::Rule :name "r2"
      :lhs (:wat::core::PersistentVector
             (:wat::core::quasiquote (:wat-tests::rete::tms::C (?k <- :k)))
             (:wat::core::quasiquote (:wat::rete::not (:wat-tests::rete::tms::B))))
      :rhs (:wat::core::PersistentVector
             (:wat::core::quasiquote (:wat-tests::rete::tms::D ?k))))))

;; ── the operation alphabet ───────────────────────────────────────────────────
;; 0 insert A0 · 1 insert A1 · 2 insert B0 · 3 retract A0 · 4 retract A1 · 5 retract B0 · 6 FIRE
;; Two A's so a retraction can leave the class non-empty (an all-or-nothing retraction cannot tell
;; "removed one" from "removed the class"), and one B so the negation can be switched on and off.
(:wat::core::defn :wat-tests::rete::tms::n-ops [] -> :wat::core::i64 7)
(:wat::core::defn :wat-tests::rete::tms::prog-len [] -> :wat::core::i64 3)

;; ── one step ─────────────────────────────────────────────────────────────────
;; `fires?` is what makes the one-shot run possible WITHOUT a separate fact model: replaying the
;; SAME program with the fire op turned into a no-op leaves exactly the multiset the interleaved
;; run ended with, so the session's own `facts` field IS the model. A hand-written model vector
;; would have to re-implement insert's append and retract's remove-all-equal, and would then be a
;; second thing that can be wrong.
(:wat::core::defn :wat-tests::rete::tms::step
  [oracle? <- :wat::core::bool  fires? <- :wat::core::bool
   s <- :wat::rete::Session  op <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::cond
    ((:wat::core::= op 0) (:wat::core::match (:wat::rete::insert s (:wat-tests::rete::tms::A 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    ((:wat::core::= op 1) (:wat::core::match (:wat::rete::insert s (:wat-tests::rete::tms::A 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    ((:wat::core::= op 2) (:wat::core::match (:wat::rete::insert s (:wat-tests::rete::tms::B 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    ((:wat::core::= op 3) (:wat::rete::retract s (:wat-tests::rete::tms::A 0)))
    ((:wat::core::= op 4) (:wat::rete::retract s (:wat-tests::rete::tms::A 1)))
    ((:wat::core::= op 5) (:wat::rete::retract s (:wat-tests::rete::tms::B 0)))
    ((:wat::core::not fires?) s)
    (oracle?               (:wat::core::match (:wat::rete::fire-rules$oracle s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))
    (:else                 (:wat::core::match (:wat::rete::fire-rules s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))))

;; The fold's carry: the session so far, and the still-undecoded tail of the program.
;; Decoding by repeated quot/rem rather than a power avoids needing `pow` and keeps each step's
;; digit derivation local — the program IS its coordinate, base `n-ops`.
(:wat::core::defrecord :wat-tests::rete::tms::Run
  [s <- :wat::rete::Session  rest <- :wat::core::i64])

(:wat::core::defn :wat-tests::rete::tms::run-prog
  [oracle? <- :wat::core::bool  fires? <- :wat::core::bool  prog <- :wat::core::i64
   s0 <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::let
    [m   (:wat-tests::rete::tms::n-ops)
     end (:wat::core::foldl
           (:wat::core::fn [acc <- :wat-tests::rete::tms::Run  _i <- :wat::core::i64]
             -> :wat-tests::rete::tms::Run
             (:wat::core::let [r  (:wat-tests::rete::tms::Run/rest acc)
                               op (:wat::core::i64::rem r m)]
               (:wat-tests::rete::tms::Run
                 :s (:wat-tests::rete::tms::step oracle? fires?
                      (:wat-tests::rete::tms::Run/s acc) op)
                 :rest (:wat::core::i64::quot r m))))
           (:wat-tests::rete::tms::Run :s s0 :rest prog)
           (:wat::core::range 0 (:wat-tests::rete::tms::prog-len)))
     ;; A FINAL fire always, so both runs end settled and the comparison is about the PATH, never
     ;; about whether the last op happened to be a fire.
     settled (:wat-tests::rete::tms::Run/s end)]
    (:wat::core::if oracle?
      (:wat::core::match (:wat::rete::fire-rules$oracle settled) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
      (:wat::core::match (:wat::rete::fire-rules settled) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))))

;; ── the queries ──────────────────────────────────────────────────────────────
;; One per un-derivation path the rules above create, so a program that breaks only one of them
;; still fails a gate.
(:wat::rete::defquery :wat-tests::rete::tms::q-C :params []
  :when [(?fact <- :wat-tests::rete::tms::C)])
(:wat::rete::defquery :wat-tests::rete::tms::q-D :params []
  :when [(?fact <- :wat-tests::rete::tms::D)])
;; `:not` over the DERIVED class — family C's shape, now under an operation program.
(:wat::rete::defquery :wat-tests::rete::tms::q-notC :params []
  :when [(:wat::rete::not (:wat-tests::rete::tms::C))])
;; An accumulate over the INSERTED class, so retraction moves the count rather than a derivation.
(:wat::rete::defquery :wat-tests::rete::tms::q-acc :params []
  :when [(?n <- (:wat::rete::acc::count) :from (:wat-tests::rete::tms::A))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 2))])

(:wat::core::defn :wat-tests::rete::tms::query-of [i <- :wat::core::i64] -> :wat::rete::Query
  (:wat::core::cond
    ((:wat::core::= i 0) (:wat-tests::rete::tms::q-C))
    ((:wat::core::= i 1) (:wat-tests::rete::tms::q-D))
    ((:wat::core::= i 2) (:wat-tests::rete::tms::q-notC))
    (:else               (:wat-tests::rete::tms::q-acc))))

(:wat::core::defn :wat-tests::rete::tms::seed [] -> :wat::rete::Session
  (:wat::rete::compile-all (:wat-tests::rete::tms::rules)
    (:wat::core::PersistentVector
      (:wat-tests::rete::tms::q-C) (:wat-tests::rete::tms::q-D)
      (:wat-tests::rete::tms::q-notC) (:wat-tests::rete::tms::q-acc))))

;; ── the case, and the four numbers ───────────────────────────────────────────
(:wat::core::defrecord :wat-tests::rete::tms::Case
  [prog <- :wat::core::i64   ;; base-`n-ops` digits, one per step — the program IS its coordinate
   q    <- :wat::core::i64])

(:wat::core::defrecord :wat-tests::rete::tms::Four
  [ni <- :wat::core::i64   ;; native, interleaved
   oi <- :wat::core::i64   ;; oracle, interleaved
   n1 <- :wat::core::i64   ;; native, one-shot over the same final multiset
   o1 <- :wat::core::i64]) ;; oracle, one-shot

(:wat::core::defn :wat-tests::rete::tms::four [c <- :wat-tests::rete::tms::Case]
  -> :wat-tests::rete::tms::Four
  (:wat::core::let [prog (:wat-tests::rete::tms::Case/prog c)
                    q    (:wat-tests::rete::tms::query-of (:wat-tests::rete::tms::Case/q c))
                    rows (:wat::core::fn [s <- :wat::rete::Session] -> :wat::core::i64
                           (:wat::core::length (:wat::rete::query s q)))]
    (:wat-tests::rete::tms::Four
      :ni (rows (:wat-tests::rete::tms::run-prog false true  prog (:wat-tests::rete::tms::seed)))
      :oi (rows (:wat-tests::rete::tms::run-prog true  true  prog (:wat-tests::rete::tms::seed)))
      :n1 (rows (:wat-tests::rete::tms::run-prog false false prog (:wat-tests::rete::tms::seed)))
      :o1 (rows (:wat-tests::rete::tms::run-prog true  false prog (:wat-tests::rete::tms::seed))))))

;; All four equal. Two INDEPENDENT claims share this gate and the coordinate tells them apart:
;;   ni != oi   → the engines disagree (the siblings' property, now under an op program)
;;   ni != n1   → NATIVE IS PATH-DEPENDENT: state survived a fire that should not have
;;   oi != o1   → the ORACLE is path-dependent, which would make the reference wrong
(:wat::core::defn :wat-tests::rete::tms::prop [c <- :wat-tests::rete::tms::Case] -> :wat::core::bool
  (:wat::core::let [f  (:wat-tests::rete::tms::four c)
                    ni (:wat-tests::rete::tms::Four/ni f)]
    (:wat::core::and
      (:wat::core::= ni (:wat-tests::rete::tms::Four/oi f))
      (:wat::core::and
        (:wat::core::= ni (:wat-tests::rete::tms::Four/n1 f))
        (:wat::core::= ni (:wat-tests::rete::tms::Four/o1 f))))))

;; ── the space ────────────────────────────────────────────────────────────────
;; EVERY program of `prog-len` steps over the `n-ops` alphabet, times every query. A flat product
;; is right here — unlike its siblings there is no per-shape parameter set, because every op is
;; legal at every position. That is the point of an operation space: the illegal orderings are
;; exactly the ones worth generating (retract before insert, fire before anything, retract twice).
(:wat::core::defn :wat-tests::rete::tms::pow-ops [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::i64::* acc (:wat-tests::rete::tms::n-ops)))
    1
    (:wat::core::range 0 (:wat-tests::rete::tms::prog-len))))

(:wat::core::defn :wat-tests::rete::tms::space []
  -> (:wat::gen::Gen :- [:wat-tests::rete::tms::Case])
  (:wat::gen::record :wat-tests::rete::tms::Case
    (:wat::gen::ints 0 (:wat-tests::rete::tms::pow-ops))
    (:wat::gen::ints 0 4)))

;; ── the gates ────────────────────────────────────────────────────────────────
;;
;; PROGRAM LENGTH IS A ONE-LINE DIAL, and the deeper setting has been RUN, not merely imagined.
;; `prog-len` 4 was measured 2026-08-27: card 9604, violations 0, 4m47s isolated (~9 min loaded).
;; That is real coverage — it reaches insert/fire/retract/fire, the shortest program that fires
;; twice with a retraction between — but it would nearly triple the whole floor for a space that
;; found nothing at length 3 either. Committed at 3 (card 1372, 39.3s isolated); turn the dial and
;; re-measure when this file has a reason to look deeper, and record the result here as this note
;; does rather than leaving the deeper run un-run.
(:wat::test::time-limit "180s")
(:wat::test::deftest :wat-tests::rete::tms::test-interleaved-ops-agree-and-are-path-independent
  (:wat::core::match (:wat::gen::check (:wat-tests::rete::tms::space) :wat-tests::rete::tms::prop)
    ((:wat::gen::CheckOutcome::Checked cases bad _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> cases 0))]
        (:wat::test::assert-eq bad 0)))
    (:wat::gen::CheckOutcome::EmptySpace (:wat::test::assert-true false))))

;; NON-VACUITY. A space where every program yields the same row count agrees with itself perfectly
;; and measures nothing — the accumulate widening in the sibling file was exactly that, and only a
;; hand-written probe caught it. Here the risk is concrete: if programs never actually changed the
;; world (every op a no-op, or the queries insensitive to them), all four numbers would be equal
;; for the most boring possible reason.
(:wat::core::defrecord :wat-tests::rete::tms::Tally
  [zero <- :wat::core::i64  nonzero <- :wat::core::i64])

;; Only the NATIVE INTERLEAVED number is needed here, not all four — running `four` would cost
;; exactly what the property above costs, doubling the file for a certificate that needs one
;; column of it.
(:wat::core::defn :wat-tests::rete::tms::rows-ni [c <- :wat-tests::rete::tms::Case] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat-tests::rete::tms::run-prog false true
        (:wat-tests::rete::tms::Case/prog c) (:wat-tests::rete::tms::seed))
      (:wat-tests::rete::tms::query-of (:wat-tests::rete::tms::Case/q c)))))

(:wat::core::defn :wat-tests::rete::tms::tally [] -> :wat-tests::rete::tms::Tally
  (:wat::core::let [g    (:wat-tests::rete::tms::space)
                    card (:wat::gen::Gen/card g)
                    at   (:wat::gen::Gen/at g)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat-tests::rete::tms::Tally  i <- :wat::core::i64]
        -> :wat-tests::rete::tms::Tally
        (:wat::core::if
          (:wat::core::= (:wat-tests::rete::tms::rows-ni (at i)) 0)
          (:wat-tests::rete::tms::Tally
            :zero (:wat::core::i64::+ (:wat-tests::rete::tms::Tally/zero acc) 1)
            :nonzero (:wat-tests::rete::tms::Tally/nonzero acc))
          (:wat-tests::rete::tms::Tally
            :zero (:wat-tests::rete::tms::Tally/zero acc)
            :nonzero (:wat::core::i64::+ (:wat-tests::rete::tms::Tally/nonzero acc) 1))))
      (:wat-tests::rete::tms::Tally :zero 0 :nonzero 0)
      (:wat::core::range 0 card))))

;; ⚠ IMMEDIATELY before the deftest, deliberately. `time-limit` is a SIBLING-FORM PRECEDING A
;; DEFTEST (`wat/test.wat`); an intervening `defn` silently drops the annotation and the test
;; falls back to the 5000ms default. This one was caught by a RED FLOOR at 5.015s after passing
;; in isolation at 4.19s — the loaded run is the only one that sees the real budget.
;; 60s, not 180s: this walk is ~4s, and a tally taking a minute genuinely IS stuck, which is the
;; deadlock-guard role the default exists for.
(:wat::test::time-limit "60s")
(:wat::test::deftest :wat-tests::rete::tms::test-programs-actually-change-the-world
  (:wat::core::let [t  (:wat-tests::rete::tms::tally)
                    z  (:wat-tests::rete::tms::Tally/zero t)
                    nz (:wat-tests::rete::tms::Tally/nonzero t)
                    ;; Some program left the query empty; some left it matching. If either arm is
                    ;; empty the ops are not reaching the queries and the gate above is theatre.
                    _  (:wat::test::assert-true (:wat::core::> z 0))
                    _  (:wat::test::assert-true (:wat::core::> nz 0))]
    ;; Reconciles with the space's own cardinality, so a truncated walk cannot pass on a prefix.
    (:wat::test::assert-eq (:wat::core::i64::+ z nz)
                           (:wat::gen::Gen/card (:wat-tests::rete::tms::space)))))
