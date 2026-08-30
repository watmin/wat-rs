;; THE FIRE DOOR of the per-session memory ceiling — derivation, with staging kept far below it.
;;
;; ⛔ THE WORKLOAD CHANGED 2026-08-29 AND THE REASON IS THE POINT. This fixture used to seed 500
;; facts through `insert` at a 4096-byte ceiling. Once `insert` began enforcing the SAME ceiling —
;; the session is the boundary, so both doors hold one contract — the very first `insert` refused,
;; and this gate started proving the INSERT door while its name and prose still claimed the fire
;; one. A control that quietly changes what it certifies has lost its power without ever failing.
;; So the workload is now one the insert door cannot catch: **400 staged facts, 40_000 derived.**
;;
;; A cross-product join — 200 `A` x 200 `B` -> 40_000 `C` — is non-cyclic, range-restricted and
;; terminating, so the verifier admits it, and it multiplies WITHIN ONE ROUND. That is exactly the
;; axis the ROUND cap cannot see: a fanout reaches the allocator while `rounds_run` is still 0
;; (measured before this ceiling existed: an abort at 6.2s, no wat error, no rule named).
;;
;; ⚠ THE CEILING IS MEASURED, NOT PICKED. Bisected 2026-08-29 on this exact workload:
;;   1 MiB · 4 MiB · 16 MiB -> refused at the FIRE door;  64 MiB · 256 MiB -> completes.
;; 16 MiB sits inside the refusing band with the 400 inserts nowhere near it, and the non-vacuity
;; row below runs the same workload at the DEFAULT ceiling, where it must complete.
(:wat::config::rete::set-max-session-bytes! 16777216)

(:wat::core::defrecord :fd::A [a <- :wat::core::i64])
(:wat::core::defrecord :fd::B [b <- :wat::core::i64])
(:wat::core::defrecord :fd::C [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::rete::defrule :fd::cross
  :when [(:fd::A (?x <- :a)) (:fd::B (?y <- :b))]
  :then [(:fd::C :a ?x :b ?y)])

(:wat::rete::defquery :fd::q :params [] :when [(?fact <- :fd::C)])

(:wat::core::defn :fd::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::insert acc (:fd::A :a i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))) (:fd::B :b i)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    s (:wat::core::range 0 200)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::rete::collect-rules :fd)
     s     (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fd::q)))
     s     (:fd::seed s)]
    ;; ⛔ THIS FIXTURE MATCHES THE ARM AND PRINTS ITS FIELDS — it is NOT codemod material, and the
    ;; codemod's generic `assertion-failed!` arms were UNDONE here on purpose. The whole point of
    ;; this gate is that a ceiling breach is a VALUE carrying `limit`, `used` and `rounds`; an arm
    ;; that collapses to a message string throws away the three numbers the gate exists to assert,
    ;; and the gate would then be proving only that *something* went wrong.
    ;;
    ;; It prints one line per field so the Rust gate can pin each exactly. The `Fired` arm prints a
    ;; count instead, which is what makes the NON-VACUITY twin meaningful: the same program under a
    ;; default ceiling takes the other arm and prints 40000.
    (:wat::core::match (:wat::rete::fire-rules s)
      ((:wat::rete::FireOutcome::Fired fired)
        (:wat::kernel::println (:wat::core::length (:wat::rete::query fired (:fd::q)))))
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded limit used rounds)
        (:wat::core::do
          (:wat::kernel::println "ARM MemoryCeilingExceeded")
          (:wat::kernel::println limit)
          (:wat::kernel::println used)
          (:wat::kernel::println rounds)))
      ((:wat::rete::FireOutcome::RoundCapExceeded cap still-deriving)
        (:wat::core::do
          (:wat::kernel::println "ARM RoundCapExceeded")
          (:wat::kernel::println cap)
          (:wat::kernel::println still-deriving))))))
