;; probe-node-share-phase-split.wat — WHERE DOES THE TIME ACTUALLY GO?
;;
;; THE QUESTION. The Clara grid times `fire-rules` and nothing else — compile and seed are
;; declared "un-timed setup" on BOTH sides (grid/node-share.wat:127, gen-node-share.sh:39-40).
;; Measured 2026-07-30 against the node-share axis, rule-count fixed at 50:
;;
;;     M      wall     fire      fire as % of wall
;;      200   0.22s    0.0068s   3.1%
;;     1000   0.87s    0.0359s   4.1%
;;     2000   3.05s    0.0770s   2.5%
;;     4000  11.91s    0.1694s   1.4%
;;
;; FIRE is LINEAR in M (0.0068 -> 0.0359 -> 0.0770 -> 0.1694, doubling as M doubles). Everything
;; ELSE is QUADRATIC: net of ~0.18s process startup, 0.04 -> 0.69 -> 2.87 -> 11.73, each doubling
;; of M multiplying it by ~4.1x. So the grid has been measuring 1-4% of the process and declaring
;; a verdict on it, and the other 96-99% grows as O(M^2).
;;
;; WHAT THIS PROBE SETTLES. `wall - fire` lumps FOUR different things together, and they have
;; completely different fixes:
;;
;;   build   — constructing N Rule records (quasiquote + splice per rule)
;;   compile — building the network from those rules (find-or-mint + wiring)
;;   seed    — 2*M `insert` calls, each conj'ing a fact and rebuilding the 7-field Session
;;   derive  — query-by-type-string -> map -> into Vector -> sort -> conj-fold to PersistentVector
;;
;; If SEED carries the quadratic, the gap is at the fact-insertion boundary — which is the door
;; every fact arriving at line rate must pass through (R25 MACHINA CHAOS DOMAT), and the fix is a
;; native insert / bulk-insert path.
;; If DERIVE carries it, the harness is materializing results by copy — R8's `reduce({}) { merge }`
;; shape — and the axes themselves are the thing to fix, not the engine.
;; Those are opposite conclusions, so guessing between them is worthless. Measure.
;;
;; WHY A SEPARATE PROBE and not a tweak to node-share.wat: that file is a MEASURED ARTIFACT whose
;; numbers are compared against Clara's. Adding timers to it changes the thing under measurement.
;; This probe copies its shape and leaves it untouched.
;;
;; FAITHFUL TO THE AXIS: build-rule / build-rules / seed / derived-vector below are copied from
;; wat-scripts/perf/grid/node-share.wat, byte-for-byte modulo the namespace (:phase:: vs :nsh::,
;; since the loader gate loads both files and the names would collide). If the axis drifts, this
;; probe stops describing it.
;;
;; SAFE: no forks, no services, single-threaded, and every size below is one the axis already ran.
;; Climb the size ladder UPWARD (a workstation died learning that); the guard in run-axis.sh does
;; NOT cover a direct invocation like this one.
;;
;; stdin : [rules items]
;; stdout: one #phase/Split EDN line
;;   echo '[50 2000]' | ./target/release/wat wat-scripts/scratch-pad/probe-node-share-phase-split.wat

(:wat::core::defrecord :phase::A   [k <- :wat::core::i64])
(:wat::core::defrecord :phase::B   [k <- :wat::core::i64])
(:wat::core::defrecord :phase::Out [k <- :wat::core::i64])

;; Split — the per-phase census. `derived-count` is the NON-VACUITY witness: if derive were
;; short-circuiting, its ns would look cheap and this count would be 0. It must equal `items`.
(:wat::core::defrecord :phase::Split
  [rules         <- :wat::core::i64
   items         <- :wat::core::i64
   build-ns      <- :wat::core::i64
   compile-ns    <- :wat::core::i64
   seed-ns       <- :wat::core::i64
   fire-ns       <- :wat::core::i64
   derive-ns     <- :wat::core::i64
   derived-count <- :wat::core::i64])

(:wat::rete::defquery :phase::q-Out
  :params []
  :when [(?fact <- :phase::Out)])


;; ── copied from grid/node-share.wat (namespace changed only) ─────────────────

(:wat::core::defn :phase::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [a-c     (:wat::core::quasiquote (:phase::A (?k <- :k)))
                    b-c     (:wat::core::quasiquote (:phase::B (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::core::= (:wat::core::unquote i)
                                  (:wat::core::i64::- ?k
                                    (:wat::core::i64::* (:wat::core::i64::/ ?k (:wat::core::unquote n)) (:wat::core::unquote n))))))
                    ins     (:wat::core::quasiquote (:phase::Out ?k))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string i)
      :lhs (:wat::core::PersistentVector a-c b-c where-c)
      :rhs (:wat::core::PersistentVector ins))))

(:wat::core::defn :phase::build-rules [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:phase::build-rule i n)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 n)))

(:wat::core::defn :phase::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert s (:phase::A i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) (:phase::B i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    session
    (:wat::core::range 0 items)))

(:wat::core::defn :phase::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  x <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::core::PersistentVector/conj acc x))
    (:wat::core::PersistentVector)
    v))

(:wat::core::defn :phase::derived-vector
  [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:phase::vec->pvec
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :wat::core::i64)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:phase::Out/k f)))
          (:wat::rete::query fired (:phase::q-Out)))))))

(:wat::core::defn :phase::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; ── main — five instants, four phases, nothing else between them ─────────────
;;
;; The `let` is strict and sequential, so each binding completes before the next instant is
;; taken; no phase can leak into a neighbour's window. Everything OUTSIDE these five instants
;; (process startup, the frozen-world load, the final println) is deliberately unmeasured —
;; the question is the split BETWEEN phases, and the wall-clock differential above already
;; bounds the fixed cost at ~0.18s.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln )
                              ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                              (:wat::kernel::ReadlnOutcome::Eof
                                (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                              (:wat::kernel::ReadlnOutcome::Stopped
                                (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    rules-n (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [rules items]")
                    items   (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [rules items]")

                    t0      (:wat::time::now)
                    rules   (:phase::build-rules rules-n)
                    t1      (:wat::time::now)
                    session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:phase::q-Out))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    t2      (:wat::time::now)
                    staged  (:phase::seed session items)
                    t3      (:wat::time::now)
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    t4      (:wat::time::now)
                    derived (:phase::derived-vector fired)
                    t5      (:wat::time::now)]
    (:wat::kernel::println
      (:phase::Split
        :rules         rules-n
        :items         items
        :build-ns      (:phase::ns-between t0 t1)
        :compile-ns    (:phase::ns-between t1 t2)
        :seed-ns       (:phase::ns-between t2 t3)
        :fire-ns       (:phase::ns-between t3 t4)
        :derive-ns     (:phase::ns-between t4 t5)
        :derived-count (:wat::core::length derived)))))
