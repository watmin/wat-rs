;; D7 — HOT-PATH COST of class-uniform batching, measured rather than asserted.
;;
;; THE INSTRUMENT FOR THE NUMBER RECORDED IN THE D7 CURE STRIKE. A metric whose
;; script is not committed cannot be re-derived, so this is the script.
;;
;; WHAT IT ISOLATES. `alpha_seed` is the pass the cure touches. The workload is a
;; single uniformly-packable class of 3-i64-field records that takes the
;; OCCUPANCY BATCH — the fast path the cure must not tax — joined against a class
;; with ZERO facts, so the fire derives nothing and the seed pass dominates what
;; is timed. `compile` and `insert-all` are outside the timing window; only
;; `fire-rules` is inside it, and the same staged session is fired REPS times
;; (value semantics: a fire does not mutate its input), so every sample measures
;; the identical work.
;;
;; WHAT THE CURE ADDS HERE, BY CONSTRUCTION. On this workload: one `bool` test per
;; batched class per seed pass, and one `bool` test at the end of the pass. Every
;; fact still takes exactly one `class_ids` probe, as before. The cost that is not
;; free — a second `class_ids` probe — falls only on facts that do NOT pack, i.e.
;; facts already committed to a full alpha-tree walk plus a compiled-cond exec,
;; and this workload deliberately has none of those so the fast path is measured
;; clean.
;;
;; Usage — prints REPS raw fire-rules nanosecond samples, ascending rep order:
;;   ./target/release/wat wat-scripts/scratch-pad/d7-seed-batch-cost.wat
;; Compare medians across two BUILDS (with and without the cure); a single build's
;; numbers are machine-relative and mean nothing on their own.

(:wat::core::defrecord :d7p::Row
  [k <- :wat::core::i64  a <- :wat::core::i64  b <- :wat::core::i64])
;; Deliberately never inserted: the join's right side stays empty so the fire
;; derives nothing and the timing window is the seed pass plus an empty join.
(:wat::core::defrecord :d7p::Other [k <- :wat::core::i64])
(:wat::core::defrecord :d7p::Hit   [k <- :wat::core::i64])

;; Both conditions are BIND-ONLY over an undiscriminated class, which is exactly
;; the shape `undiscriminated_leaves` + `bind_only` admits to the occupancy batch.
(:wat::rete::defrule :d7p::r
  :when  [(:d7p::Row   (?k <- :k))
          (:d7p::Other (?k <- :k))]
  :then  [(:d7p::Hit ?k)])

(:wat::rete::defquery :d7p::q :params [] :when [(?fact <- :d7p::Hit)])

(:wat::core::defn :d7p::rows
  [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::into (:wat::core::Vector :wat::core::Record)
      (:wat::core::map
        (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::Record
          (:d7p::Row :k i :a i :b i))
        (:wat::core::range 0 n)))))

(:wat::core::defn :d7p::staged [n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
      (:wat::core::match (:wat::rete::compile-all
          (:wat::core::PersistentVector (:d7p::r))
          (:wat::core::PersistentVector (:d7p::q)))
        ((:wat::rete::CompileOutcome::Compiled __s) __s)
        ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
          (:wat::kernel::assertion-failed! "compile" :wat::core::None :wat::core::None)))
      (:d7p::rows n))
    ((:wat::rete::InsertOutcome::Inserted __s) __s)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
      (:wat::kernel::assertion-failed! "insert" :wat::core::None :wat::core::None))))

(:wat::core::defn :d7p::fire-ns [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::let
    [t0 (:wat::time::now)
     fired (:wat::core::match (:wat::rete::fire-rules s)
             ((:wat::rete::FireOutcome::Fired __f) __f)
             ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
               (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))
             ((:wat::rete::FireOutcome::RoundCapExceeded __c __x)
               (:wat::kernel::assertion-failed! "cap" :wat::core::None :wat::core::None)))
     ;; Force the result so the fire cannot be elided or deferred out of the window.
     n (:wat::core::Vector/length
         (:wat::core::into (:wat::core::Vector :wat::core::PersistentMap)
           (:wat::rete::query fired (:d7p::q))))
     t1 (:wat::time::now)]
    (:wat::core::i64::+
      (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0))
      n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s (:d7p::staged 200000)
     samples (:wat::core::into (:wat::core::Vector :wat::core::i64)
               (:wat::core::map
                 (:wat::core::fn [__i <- :wat::core::i64] -> :wat::core::i64 (:d7p::fire-ns s))
                 (:wat::core::range 0 9)))]
    (:wat::kernel::println (:wat::core::into (:wat::core::PersistentVector) samples))))
