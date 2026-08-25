;; bench-118B5-into-stream-vs-native-concat.wat — is `into` over a Stream the last INTERPRETED
;; drain in the collection surface?
;;
;; ⛔ WHY: `into` is a five-arm defclause (wat/seq.wat:166). Every EAGER arm is ONE NATIVE call —
;; `concat`, `PersistentVector/concat`. The two STREAM arms delegate to `stream->vec` /
;; `stream->pvec`, which are hand-rolled wat `next`-walks conj-ing one element at a time: N
;; interpreted steps against 1 native call for the same materialization.
;;
;; Stone 118.B1 sketched B5 as "into absorbs the drain; stream->pvec / stream->vec deleted",
;; assuming those verbs were redundant PUBLIC siblings of `into`. They are not — they are `into`'s
;; own IMPLEMENTATION. So the question is not whether to delete them but whether the drain deserves
;; a native kernel with the wat kept as its oracle, the shape B6 gave `foldl` and B4-0 gave `nth`.
;;
;; This bench answers ONE question and no other: at the same n, how does materializing a STREAM
;; compare with materializing an eager VECTOR through the native concat arm? It does NOT claim the
;; two are semantically interchangeable — a Stream must be forced and a Vector must not — which is
;; exactly why the gap is the interpreted-walk cost and not a like-for-like speedup.
;;
;; ⚠ THE NON-VACUITY GUARD IS THE LENGTHS: both paths must land the same element count, printed.
;; A drain that silently short-circuited would otherwise post a flattering number.
;;
;; RUN (capped, per the standing rule):
;;   systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 timeout 300 \
;;     ./target/release/wat wat-scripts/scratch-pad/bench-118B5-into-stream-vs-native-concat.wat

(:wat::core::defn :bench::ns [t0 <- :wat::time::Instant t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; STREAM path — `map` is lazy, so this is the two-arm interpreted drain under test.
(:wat::core::defn :bench::stream-drain
  [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) v))))

;; EAGER path — the native PersistentVector/concat arm, same n elements materialized.
(:wat::core::defn :bench::native-concat
  [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::into (:wat::core::PersistentVector) v)))

;; DRAIN-ONLY path — `Seqable/seq` yields a (Stream :- [T]) over the SAME vector with NO user closure
;; anywhere. This is the control that separates `map`'s per-element interpreted closure call from
;; the drain itself. Without it, "stream is 49x slower" is a claim about a component, read rather
;; than measured. [[feedback_measure_the_decomposition_never_read_it]]
(:wat::core::defn :bench::drain-only
  [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::Seqable/seq v))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n  200000
     v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::range 0 n))
     a0 (:wat::time::now) ra (:bench::stream-drain v)  a1 (:wat::time::now)
     b0 (:wat::time::now) rb (:bench::native-concat v) b1 (:wat::time::now)
     c0 (:wat::time::now) rc (:bench::native-concat v) c1 (:wat::time::now)
     d0 (:wat::time::now) rd (:bench::stream-drain v)  d1 (:wat::time::now)
     e0 (:wat::time::now) re (:bench::drain-only v)    e1 (:wat::time::now)
     f0 (:wat::time::now) rf (:bench::drain-only v)    f1 (:wat::time::now)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "n={n} NONVACUITY map+drain={ra}/{rd} native={rb}/{rc} drain-only={re}/{rf} | map+drain={ad}/{dd}ms | native={bd}/{cd}ms | DRAIN-ONLY={ed}/{fd}ms"
        :n n :ra ra :rb rb :rc rc :rd rd :re re :rf rf
        :ed (:wat::core::i64::/ (:bench::ns e0 e1) 1000000)
        :fd (:wat::core::i64::/ (:bench::ns f0 f1) 1000000)
        :ad (:wat::core::i64::/ (:bench::ns a0 a1) 1000000)
        :bd (:wat::core::i64::/ (:bench::ns b0 b1) 1000000)
        :cd (:wat::core::i64::/ (:bench::ns c0 c1) 1000000)
        :dd (:wat::core::i64::/ (:bench::ns d0 d1) 1000000)))))
