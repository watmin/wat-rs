;; probe-pv-lazy-materialize-cost.wat — IS MATERIALISING A LAZY PIPELINE INTO A PersistentVector
;; QUADRATIC? A pure-collections measurement: NO rete, no engine, no harness.
;;
;; THE QUESTION. probe-derive-chain-split.wat (2026-07-30) found the node-share axis's O(M^2) is
;; 99.5% inside `query-by-type-string`, whose last expression is
;;   (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter pred all))
;; Everything around it — map, into(Vector), sort, a conj-fold — measured linear.
;;
;; A CANDIDATE MECHANISM, grounded but NOT proven to be on this path: `rest` on a
;; PersistentVector is O(n) BY CONSTRUCTION (collection/eval.rs:1643-1655 — it allocates a fresh
;; VectorSync and push_back-clones every remaining element). And it has no choice: rpds 1.2.1's
;; Vector exposes `drop_last` / `push_back` and has NO `drop_first` and NO `push_front` — a
;; bitmapped trie is efficient at the TAIL and has no head operation at all. So a walk built из
;; repeated `rest` is O(n^2), and the primitive cannot be made cheap without changing the
;; representation.
;;
;; BUT the link is UNPROVEN: `stream->pvec`'s own `rest` calls hit the STREAM arm, which is
;; `Arc::clone(tail)` — O(1). For the O(n) `rest` to be the mechanism it must sit inside how the
;; lazy `filter` steps its EAGER PersistentVector source, which I have not read. Three of my
;; hypotheses have already been wrong today (insertion; `into Vector`; a partial read that
;; "cleared" query-by-type-string). So this probe MEASURES instead of arguing.
;;
;; THE FOUR COLUMNS, all over the SAME PersistentVector of n i64s with an all-pass predicate
;; (matching the rete case, where the filter keeps every fact of the queried type):
;;   into-pv    (into (PersistentVector) (filter pred pv))  — THE SUSPECT
;;   into-vec   (into (Vector)           (filter pred pv))  — same pipeline, other target
;;   fold       (foldl conj-if-pred pv)                     — CONTROL, no laziness at all
;;   rest-walk  an explicit first/rest recursion over pv    — the `rest` mechanism, ISOLATED
;;
;; HOW TO READ IT. `rest-walk` is the decisive column: it exercises repeated PersistentVector
;; `rest` and NOTHING else. If rest-walk is quadratic AND into-pv is quadratic, the mechanism is
;; identified. If rest-walk is quadratic but into-pv is LINEAR, then `rest` is a real defect that
;; is NOT on this path and the query quadratic is still unexplained — a different probe follows.
;; If NEITHER is quadratic at these sizes, the cost is elsewhere and this whole line dies here.
;;
;; SAFE: pure collections, single-threaded, no forks, no services. Sizes match the ladder already
;; run. Climb UPWARD.
;;
;; stdin : [n]
;; stdout: one #cx/Costs EDN line
;;   echo '[2000]' | ./target/release/wat wat-scripts/scratch-pad/probe-pv-lazy-materialize-cost.wat

(:wat::core::defrecord :cx::Costs
  [n            <- :wat::core::i64
   build-ns     <- :wat::core::i64
   into-pv-ns   <- :wat::core::i64
   into-vec-ns  <- :wat::core::i64
   fold-ns      <- :wat::core::i64
   rest-walk-ns <- :wat::core::i64
   into-pv-len  <- :wat::core::i64
   into-vec-len <- :wat::core::i64
   fold-len     <- :wat::core::i64
   rest-walk-sum <- :wat::core::i64])

(:wat::core::defn :cx::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; keep? — the all-pass predicate. Matches query-by-type-string's real behaviour on this
;; workload: every production fact IS of the queried type, so its filter keeps 100%. A
;; selective predicate would shrink the output and confound a size comparison.
(:wat::core::defn :cx::keep? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::>= x 0))

;; rest-walk — the `rest` mechanism ISOLATED. Sums the vector by first/rest recursion, which is
;; the shape a lazy walk over an eager container would take if it stepped via `rest`. Guarded on
;; `empty?` because `rest` RAISES on an empty PersistentVector (collection/eval.rs:1646).
;; Returns the sum purely as a non-vacuity witness — the walk must actually visit every element.
(:wat::core::defn :cx::rest-walk
  [pv <- (:wat::core::PersistentVector :- [:wat::core::i64])  acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? pv)
    acc
    (:cx::rest-walk (:wat::core::rest pv) (:wat::i64::+ acc (:wat::core::first pv)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params (:wat::core::match (:wat::kernel::readln )
                             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                             (:wat::kernel::ReadlnOutcome::Eof
                               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                             (:wat::kernel::ReadlnOutcome::Stopped
                               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    n      (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [n]")

                    b0     (:wat::time::now)
                    pv     (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])
                                              i   <- :wat::core::i64]
                               -> (:wat::core::PersistentVector :- [:wat::core::i64])
                               (:wat::vector::conj acc i))
                             (:wat::core::PersistentVector)
                             (:wat::core::range 0 n))
                    b1     (:wat::time::now)

                    ;; THE SUSPECT — lazy filter materialised into a PersistentVector.
                    p0     (:wat::time::now)
                    ipv    (:wat::core::into (:wat::core::PersistentVector)
                             (:wat::core::filter :cx::keep? pv))
                    p1     (:wat::time::now)

                    ;; Same pipeline, Vector target — measured linear in the derive chain.
                    v0     (:wat::time::now)
                    ivec   (:wat::core::into (:wat::core::Vector :wat::core::i64)
                             (:wat::core::filter :cx::keep? pv))
                    v1     (:wat::time::now)

                    ;; CONTROL — no laziness anywhere; foldl iterates the PV natively.
                    f0     (:wat::time::now)
                    folded (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])
                                              x   <- :wat::core::i64]
                               -> (:wat::core::PersistentVector :- [:wat::core::i64])
                               (:wat::core::if (:cx::keep? x)
                                 (:wat::vector::conj acc x)
                                 acc))
                             (:wat::core::PersistentVector)
                             pv)
                    f1     (:wat::time::now)

                    ;; THE MECHANISM, ISOLATED — repeated PersistentVector `rest`, nothing else.
                    r0     (:wat::time::now)
                    rsum   (:cx::rest-walk pv 0)
                    r1     (:wat::time::now)]
    (:wat::kernel::println
      (:cx::Costs
        :n             n
        :build-ns      (:cx::ns-between b0 b1)
        :into-pv-ns    (:cx::ns-between p0 p1)
        :into-vec-ns   (:cx::ns-between v0 v1)
        :fold-ns       (:cx::ns-between f0 f1)
        :rest-walk-ns  (:cx::ns-between r0 r1)
        :into-pv-len   (:wat::core::length ipv)
        :into-vec-len  (:wat::core::length ivec)
        :fold-len      (:wat::core::length folded)
        :rest-walk-sum rsum))))
