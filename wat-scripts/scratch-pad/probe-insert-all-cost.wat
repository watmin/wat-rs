;; probe-insert-all-cost.wat — THE MEASUREMENT the insert-all stone's DoD asks for: seed 40,000
;; facts via ONE `insert-all` call vs the existing `foldl` + `insert` (N single-fact calls, the
;; pre-existing streaming pattern), expecting roughly a 41 ms drop (DESIGN-STONE-insert-all.md's
;; measured ~1.03 µs/fact of pure Session-rebuild cost above a bare `conj`, times 40,000).
;;
;; Both arms construct exactly `n` fresh Reading records — the SAME work `probe-insert-cost-
;; split.wat` isolated as the harness floor — so the only thing that can differ between them is
;; N Session rebuilds (`insert` × n) vs ONE Session rebuild (`insert-all` once). This is a
;; SUBTRACTION of a measured per-fact cost, not a model: the number decides, not the arithmetic.
;;
;; NON-VACUITY: both arms' resulting `facts` length must equal `n` — a short-circuited or
;; optimised-away arm would look fast AND drop this witness.
;;
;; SAFE: pure fold + one Session, single-threaded, no forks, no services. Still climb the ladder
;; UPWARD under the guard — a timeout bounds time, not RAM:
;;   systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0 -- \
;;     timeout 120 ./target/release/wat wat-scripts/scratch-pad/probe-insert-all-cost.wat
;;
;; stdin : [n]
;; stdout: one #iac/Cost EDN line
;;   echo '[40000]' | ./target/release/wat wat-scripts/scratch-pad/probe-insert-all-cost.wat

(:wat::core::defrecord :iac::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :iac::Out     [g <- :wat::core::i64])

;; One rule, so the Session under test has a real compiled network rather than an empty one.
;; `insert`/`insert-all` perform ZERO activation (wat/rete.wat:828-830 — the WM stays open until
;; fire-rules), so the network's size must not affect the per-insert cost.
(:wat::rete::defrule :iac::pass-rule
  :when
  [(:iac::Reading (?g <- :g))]
  :then
  [(:iac::Out ?g)])

(:wat::core::defrecord :iac::Cost
  [n              <- :wat::core::i64
   chained-ns     <- :wat::core::i64   ;; n × construct + foldl + insert (2-ary, N rebuilds)
   batch-ns       <- :wat::core::i64   ;; n × construct + conj, then ONE insert-all (1 rebuild)
   drop-ns        <- :wat::core::i64   ;; chained-ns - batch-ns (the win)
   chained-len    <- :wat::core::i64   ;; witness: must equal n
   batch-len      <- :wat::core::i64]) ;; witness: must equal n

(:wat::core::defn :iac::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; ── arm 1 — the existing hot path: construct + 2-ary insert, one fact at a time ──────────────
(:wat::core::defn :iac::seed-chained [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::core::match (:wat::rete::insert s (:iac::Reading :g 0 :v i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    session
    (:wat::core::range 0 n)))

;; ── arm 2 — construct + conj into a vector, then ONE insert-all call ─────────────────────────
(:wat::core::defn :iac::seed-batch [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::let [facts (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:iac::Reading])  i <- :wat::core::i64]
                               -> (:wat::core::PersistentVector :- [:iac::Reading])
                               (:wat::core::PersistentVector/conj acc (:iac::Reading :g 0 :v i)))
                             (:wat::core::PersistentVector)
                             (:wat::core::range 0 n))]
    (:wat::core::match (:wat::rete::insert-all session facts) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params (:wat::core::match (:wat::kernel::readln )
                             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                             (:wat::kernel::ReadlnOutcome::Eof
                               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                             (:wat::kernel::ReadlnOutcome::Stopped
                               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    n       (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [n]")

                    ;; Two independent compiled sessions, compiled OUTSIDE every timed window.
                    session-a (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :iac)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                    session-b (:wat::core::match (:wat::rete::compile (:wat::rete::collect-rules :iac)) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))

                    t0      (:wat::time::now)
                    sa      (:iac::seed-chained session-a n)
                    t1      (:wat::time::now)

                    t2      (:wat::time::now)
                    sb      (:iac::seed-batch session-b n)
                    t3      (:wat::time::now)

                    chained-ns (:iac::ns-between t0 t1)
                    batch-ns   (:iac::ns-between t2 t3)]
    (:wat::kernel::println
      (:iac::Cost
        :n            n
        :chained-ns   chained-ns
        :batch-ns     batch-ns
        :drop-ns      (:wat::core::i64::- chained-ns batch-ns)
        :chained-len  (:wat::core::length (:wat::rete::Session/facts sa))
        :batch-len    (:wat::core::length (:wat::rete::Session/facts sb))))))
