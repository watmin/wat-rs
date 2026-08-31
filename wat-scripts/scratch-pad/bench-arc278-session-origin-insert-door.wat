;; BENCH — arc 278, the session-ceiling strike: what the INSERT DOOR costs per fact.
;;
;; ★ WHAT THIS IS FOR. `alloc_counter::session_bytes` is read once per `insert` call, at the door
;; in `rete/kernel/insert.rs`. Its store used to be a single `const`-initialised
;; `Cell<Option<usize>>`; the session-origin strike replaced it with a per-session store, and
;; `alloc_counter`'s own doc warns that this slot is on the insert hot path and was `const`-init
;; for exactly that reason. This bench is the instrument that turns "is that free?" into a number.
;;
;; It measures the WHOLE door (fact construction + `PersistentVector/conj` + the 8-field `Session`
;; rebuild + the ceiling read), because that is the thing a user pays. `session_bytes` is a small
;; fraction of it — which is the point: the question is whether the replacement is visible AT THE
;; DOOR, not in isolation. Run it on the binary before and after the change and compare ns/fact.
;;
;; Shape discipline (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`): fixed n, three
;; blocks per process so within-run spread is visible, and a non-vacuity row proving every block
;; actually staged n facts rather than tripping the ceiling and short-circuiting.
;;
;; The ceiling is set ABOVE anything this can reach: a breach would short-circuit the fold and the
;; timing would be measuring a refusal, not a door.
(:wat::config::rete::set-max-session-bytes! 400000000000)

(:wat::core::defrecord :bd::Edge [a <- :wat::core::i64  b <- :wat::core::i64])
(:wat::rete::defrule :bd::noop :when [(:bd::Edge (?a <- :a))] :then [])

(:wat::core::defn :bd::compile [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :bd) (:wat::core::PersistentVector))
    ((:wat::rete::CompileOutcome::Compiled __s) __s)
    ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
      (:wat::kernel::assertion-failed! "bench: the rule set may not terminate" :wat::core::None :wat::core::None))))

;; Stage `n` facts through the single-fact door, short-circuiting on a ceiling so a breach cannot
;; masquerade as a fast run.
(:wat::core::defn :bd::stage [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::InsertOutcome
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::InsertOutcome  i <- :wat::core::i64] -> :wat::rete::InsertOutcome
      (:wat::core::match acc
        ((:wat::rete::InsertOutcome::Inserted session)
          (:wat::rete::insert session (:bd::Edge :a i :b (:wat::core::i64::+ i 1))))
        ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __s) acc)))
    (:wat::rete::InsertOutcome::Inserted s)
    (:wat::core::range 0 n)))

;; NON-VACUITY: how many facts the block actually staged. `-1` means it refused, and any timing
;; from that block is measuring the wrong thing.
(:wat::core::defn :bd::staged [o <- :wat::rete::InsertOutcome] -> :wat::core::i64
  (:wat::core::match o
    ((:wat::rete::InsertOutcome::Inserted s) (:wat::core::length (:wat::rete::Session/facts s)))
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __st) -1)))

(:wat::core::defn :bd::ns [t0 <- :wat::time::Instant t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n  20000
     a0 (:wat::time::now) ra (:bd::stage (:bd::compile) n) a1 (:wat::time::now)
     b0 (:wat::time::now) rb (:bd::stage (:bd::compile) n) b1 (:wat::time::now)
     c0 (:wat::time::now) rc (:bd::stage (:bd::compile) n) c1 (:wat::time::now)]
    (:wat::kernel::println
      (:wat::core::string::interpolate
        "n={n} STAGED a={sa} b={sb} c={sc} | ns/fact a={an} b={bn} c={cn}"
        :n n :sa (:bd::staged ra) :sb (:bd::staged rb) :sc (:bd::staged rc)
        :an (:wat::core::i64::/ (:bd::ns a0 a1) n)
        :bn (:wat::core::i64::/ (:bd::ns b0 b1) n)
        :cn (:wat::core::i64::/ (:bd::ns c0 c1) n)))))
