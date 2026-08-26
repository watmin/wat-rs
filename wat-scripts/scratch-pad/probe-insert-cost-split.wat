;; probe-insert-cost-split.wat — OF THE ~15 µs PER FACT, HOW MUCH IS `insert` AND HOW MUCH IS THE
;; HARNESS AROUND IT?
;;
;; THE QUESTION. `probe-accumulate-gather-cost.wat` measured seeding at 15.2 µs/fact — the same
;; constant at 8,400 facts and at 20,100 facts, so it is LINEAR, not a quadratic to hunt. At the
;; grid's own accum size it is 306ms of a 412ms workload: 74%, and three times the fire. That makes
;; it the gate on line rate (R25 MACHINA CHAOS DOMAT), and the next thing to cut.
;;
;; The tempting root, grounded to file:line but NOT yet proven to dominate: `:wat::rete::insert`
;; (wat/rete.wat:833-844) is INTERPRETED WAT, and per call it does
;;   6 Session accessors + 1 more for :facts + 1 PersistentVector/conj + a 7-field Session ctor
;; and its own header says the reconstruction exists to satisfy the CHECKER ("Record/assoc returns
;; the base :wat::core::Record type; the typed Session constructor preserves the concrete return
;; type"), not because the semantics need it.
;;
;; ⛔ WHY THAT IS NOT ENOUGH TO ACT ON. The seed loop is ALSO interpreted, and R24's own do-not
;; says it outright: "the strat-neg harness is O(n^2) INTERPRETED — big runs are HARNESS-THEATER,
;; not fire cost." Per fact the loop pays for a `foldl` step, a closure call, AND constructing the
;; Reading record — before `insert` is even entered. If that floor is 14 of the 15 µs, then making
;; `insert` native buys ~7% and a strike aimed at it is aimed at the wrong thing.
;;
;; So this probe DECOMPOSES rather than confirms. Three arms, each folding over the SAME range and
;; constructing the SAME record the same number of times, differing only in what they do with it:
;;
;;   baseline  fold + construct + read a field (sum it)      — the interpreted HARNESS FLOOR
;;   conj      fold + construct + PersistentVector/conj      — floor + the container
;;   insert    fold + construct + :wat::rete::insert         — floor + the real thing
;;
;; The subtractions are the answer, and they are only valid because every arm constructs exactly
;; `n` records:
;;   harness floor          = baseline
;;   container cost         = conj   - baseline
;;   insert's OWN cost      = insert - conj
;;
;; HOW TO READ IT — and how it can KILL the hypothesis. If `insert - conj` is the bulk of the
;; per-fact cost, `insert` is the target and a native insert / bulk-insert path is the stone. If
;; `baseline` is the bulk, then the HARNESS is the cost, `insert` is nearly free, and the honest
;; finding is that the seed number never measured the engine at all — in which case do NOT brief a
;; native insert on the strength of it. Either way the number decides, not the reading of
;; rete.wat:833.
;;
;; NON-VACUITY. Each arm emits its own witness and all three must equal `n`: `baseline-sum` is the
;; sum 0+1+…+(n-1) (emitted alongside its expected value), `conj-len` is the built vector's length,
;; `insert-len` is the Session's fact count. An arm that got optimised or short-circuited away
;; would look free AND drop its witness, so a fast number with a wrong witness voids the run.
;;
;; SAFE: pure fold + one Session, single-threaded, no forks, no services. Still climb the ladder
;; UPWARD under the guard — a timeout bounds time, not RAM:
;;   systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0 -- \
;;     timeout 120 ./target/release/wat wat-scripts/scratch-pad/probe-insert-cost-split.wat
;;
;; stdin : [n]
;; stdout: one #ins/Split EDN line
;;   echo '[20000]' | ./target/release/wat wat-scripts/scratch-pad/probe-insert-cost-split.wat

(:wat::core::defrecord :ins::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :ins::Out     [g <- :wat::core::i64])

;; One rule, so the Session under test has a real compiled network rather than an empty one.
;; `insert` performs ZERO activation (wat/rete.wat:828-830 — the WM stays open until fire-rules),
;; so the network's size must not affect the per-insert cost; including a rule keeps the shape
;; honest without claiming the network is exercised.
(:wat::rete::defrule :ins::pass-rule
  :when
  [(:ins::Reading (?g <- :g))]
  :then
  [(:ins::Out ?g)])

(:wat::core::defrecord :ins::Split
  [n                <- :wat::core::i64
   baseline-ns      <- :wat::core::i64   ;; fold + construct + read a field
   conj-ns          <- :wat::core::i64   ;; fold + construct + PersistentVector/conj
   insert-prime-ns  <- :wat::core::i64   ;; fold + construct + :wat::rete::insert' (native prime)
   insert-ns        <- :wat::core::i64   ;; fold + construct + :wat::rete::insert (public defclause)
   baseline-sum     <- :wat::core::i64   ;; witness: must equal expected-sum
   expected-sum     <- :wat::core::i64
   conj-len         <- :wat::core::i64   ;; witness: must equal n
   insert-prime-len <- :wat::core::i64   ;; witness: must equal n
   insert-len       <- :wat::core::i64]) ;; witness: must equal n

(:wat::core::defn :ins::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; ── arm 1 — the interpreted harness floor ────────────────────────────────────
;; Constructs the record and READS A FIELD back, so the construction cannot be elided and the
;; measurement is of real work. Returns the running sum as its own witness.
(:wat::core::defn :ins::baseline [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ acc (:ins::Reading/v (:ins::Reading :g 0 :v i))))
    0
    (:wat::core::range 0 n)))

;; ── arm 2 — floor + the persistent container ─────────────────────────────────
(:wat::core::defn :ins::conj-only [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:ins::Reading])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:ins::Reading])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:ins::Reading])
      (:wat::core::PersistentVector/conj acc (:ins::Reading :g 0 :v i)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 n)))

;; ── arm 3 — floor + the native prime `insert'` ───────────────────────────────
(:wat::core::defn :ins::insert-prime [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert s (:ins::Reading :g 0 :v i)))
    session
    (:wat::core::range 0 n)))

;; ── arm 4 — floor + the public `insert` (defclause → insert') ────────────────
(:wat::core::defn :ins::insert-all [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert s (:ins::Reading :g 0 :v i)))
    session
    (:wat::core::range 0 n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params (:wat::core::match (:wat::kernel::readln )
                             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                             (:wat::kernel::ReadlnOutcome::Eof
                               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                             (:wat::kernel::ReadlnOutcome::Stopped
                               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    n       (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [n]")

                    ;; The Session is compiled OUTSIDE every timed window — compile is not under test.
                    session (:wat::rete::compile (:wat::rete::collect-rules :ins))

                    b0      (:wat::time::now)
                    bsum    (:ins::baseline n)
                    b1      (:wat::time::now)

                    c0      (:wat::time::now)
                    cv      (:ins::conj-only n)
                    c1      (:wat::time::now)

                    p0      (:wat::time::now)
                    primed  (:ins::insert-prime session n)
                    p1      (:wat::time::now)

                    i0      (:wat::time::now)
                    staged  (:ins::insert-all session n)
                    i1      (:wat::time::now)]
    (:wat::kernel::println
      (:ins::Split
        :n            n
        :baseline-ns  (:ins::ns-between b0 b1)
        :conj-ns      (:ins::ns-between c0 c1)
        :insert-prime-ns (:ins::ns-between p0 p1)
        :insert-ns    (:ins::ns-between i0 i1)
        :baseline-sum bsum
        ;; 0+1+…+(n-1) = n(n-1)/2 — computed, never eyeballed against a magic number.
        :expected-sum (:wat::i64::/ (:wat::i64::* n (:wat::i64::- n 1)) 2)
        :conj-len     (:wat::core::length cv)
        :insert-prime-len (:wat::core::length (:wat::rete::Session/facts primed))
        :insert-len   (:wat::core::length (:wat::rete::Session/facts staged))))))
