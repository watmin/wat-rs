;; probe-derive-chain-split.wat — WHICH LINK OF THE DERIVE CHAIN IS QUADRATIC?
;;
;; THE QUESTION. probe-node-share-phase-split.wat (2026-07-30) split the node-share axis into its
;; phases, rule-count fixed at 50:
;;
;;     M      build   compile     seed      fire      derive
;;      200   0.3ms     9.1ms     4.6ms     7.9ms      26.3ms
;;     1000   0.3ms    11.3ms    32.7ms    47.6ms     639.3ms
;;     2000   0.3ms     7.9ms    51.1ms    75.3ms    2720.9ms
;;     4000   0.3ms     9.0ms   100.7ms   166.4ms   11697.4ms
;;
;; build/compile FLAT, seed and fire LINEAR — the engine is healthy. DERIVE is O(M^2): each
;; doubling of M multiplies it by ~4.3x. At M=4000 the whole engine costs 276ms and materialising
;; the answer costs 11.7 SECONDS, 42x longer than computing it.
;;
;; The derive chain is five links:
;;   query-by-type-string -> map -> into(Vector) -> sort -> conj-fold to PersistentVector
;; `query-by-type-string` is already cleared: it is a nested foldl of PersistentVector/conj
;; (wat/rete.wat:1891-1899), O(n log n). So the quadratic is in one of the other four.
;;
;; WHY IT MATTERS BEYOND RETE. The prime suspect is `into (:wat::core::Vector ...)` being
;; immutable-BY-COPY — R8's `reduce({}) { merge }` shape, where every append clones the
;; accumulator. If that is it, this is NOT a rete defect at all: it is every piece of wat user
;; code that materialises a collection by folding, and this harness merely happened to expose it.
;; That is a substrate-wide finding, and it is why this probe exists rather than a harness patch.
;;
;; THE CONTROL (the load-bearing half). Timing the four links alone cannot distinguish "Vector
;; append is O(n)" from "the elements are just expensive" — and `map` may be LAZY, in which case
;; its cost hides inside whatever forces it and the per-link numbers mislead. So the probe also
;; runs a DIRECT path over the SAME query result: one foldl conj'ing straight into a
;; PersistentVector, no Vector anywhere, no map. Same input, same output cardinality, same work
;; modulo the container. If :direct is linear while :into is quadratic, the container is the
;; defect and the elements are exonerated. If BOTH are quadratic, the suspicion is wrong and the
;; cost is in producing the elements — measure again rather than believe this note.
;;
;; SAFE: no forks, no services, single-threaded, sizes already run by the axis. Climb UPWARD.
;;
;; stdin : [rules items]
;; stdout: one #derive/Links EDN line
;;   echo '[50 2000]' | ./target/release/wat wat-scripts/scratch-pad/probe-derive-chain-split.wat

(:wat::core::defrecord :dc::A   [k <- :wat::core::i64])
(:wat::core::defrecord :dc::B   [k <- :wat::core::i64])
(:wat::core::defrecord :dc::Out [k <- :wat::core::i64])

;; Links — per-link nanoseconds plus the cardinality witnesses.
;; The three counts are the NON-VACUITY guard: every one must equal `items`. A link that
;; short-circuited would look cheap AND drop its count, so a fast number with a right count is
;; the only reading that means anything.
(:wat::core::defrecord :dc::Links
  [rules        <- :wat::core::i64
   items        <- :wat::core::i64
   query-ns     <- :wat::core::i64
   map-ns       <- :wat::core::i64
   into-ns      <- :wat::core::i64
   sort-ns      <- :wat::core::i64
   topv-ns      <- :wat::core::i64
   direct-ns    <- :wat::core::i64   ;; the CONTROL: query -> foldl conj -> PersistentVector
   query-count  <- :wat::core::i64
   into-count   <- :wat::core::i64
   direct-count <- :wat::core::i64])

(:wat::rete::defquery :dc::q-Out
  :params []
  :when [(?fact <- :dc::Out)])


;; ── the workload, copied from grid/node-share.wat (namespace changed only) ───

(:wat::core::defn :dc::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [a-c     (:wat::core::quasiquote (:dc::A (?k <- :k)))
                    b-c     (:wat::core::quasiquote (:dc::B (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::core::= (:wat::core::unquote i)
                                  (:wat::i64::- ?k
                                    (:wat::i64::* (:wat::i64::/ ?k (:wat::core::unquote n)) (:wat::core::unquote n))))))
                    ins     (:wat::core::quasiquote (:dc::Out ?k))]
    (:wat::rete::Rule :name (:wat::i64::to-string i)
      :lhs (:wat::core::PersistentVector a-c b-c where-c)
      :rhs (:wat::core::PersistentVector ins))))

(:wat::core::defn :dc::build-rules [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:dc::build-rule i n)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 n)))

(:wat::core::defn :dc::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert (:wat::rete::insert s (:dc::A i)) (:dc::B i)))
    session
    (:wat::core::range 0 items)))

(:wat::core::defn :dc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; ── main — one fire, then the derive chain link by link, then the control ────
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln )
                              ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                              (:wat::kernel::ReadlnOutcome::Eof
                                (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                              (:wat::kernel::ReadlnOutcome::Stopped
                                (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    rules-n (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [rules items]")
                    items   (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [rules items]")
                    staged  (:dc::seed (:wat::rete::compile-all (:dc::build-rules rules-n) (:wat::core::PersistentVector (:dc::q-Out))) items)
                    fired   (:wat::rete::fire-rules staged)

                    ;; ── the chain as the axis writes it, link by link ────────
                    q0      (:wat::time::now)
                    q       (:wat::rete::query fired (:dc::q-Out))
                    q1      (:wat::time::now)
                    mapped  (:wat::core::map
                              (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:dc::Out/k f)))
                              q)
                    q2      (:wat::time::now)
                    vec     (:wat::core::into (:wat::core::Vector :wat::core::i64) mapped)
                    q3      (:wat::time::now)
                    sorted  (:wat::core::sort vec)
                    q4      (:wat::time::now)
                    pv      (:wat::core::foldl
                              (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])
                                               x   <- :wat::core::i64]
                                -> (:wat::core::PersistentVector :- [:wat::core::i64])
                                (:wat::core::PersistentVector/conj acc x))
                              (:wat::core::PersistentVector)
                              sorted)
                    q5      (:wat::time::now)

                    ;; ── the CONTROL: same query result, PersistentVector only ─
                    ;; No Vector, no map — one foldl that reads the accessor and conj's. If this
                    ;; is linear while the chain above is quadratic, the container is the defect.
                    d0      (:wat::time::now)
                    direct  (:wat::core::foldl
                              (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])
                                               p   <- :wat::core::PersistentMap]
                                -> (:wat::core::PersistentVector :- [:wat::core::i64])
                                (:wat::core::let [f (:wat::core::Option/expect
                                                      (:wat::core::PersistentMap/get p "?fact")
                                                      "q-Out: ?fact")]
                                  (:wat::core::PersistentVector/conj acc (:dc::Out/k f))))
                              (:wat::core::PersistentVector)
                              q)
                    d1      (:wat::time::now)]
    (:wat::kernel::println
      (:dc::Links
        :rules        rules-n
        :items        items
        :query-ns     (:dc::ns-between q0 q1)
        :map-ns       (:dc::ns-between q1 q2)
        :into-ns      (:dc::ns-between q2 q3)
        :sort-ns      (:dc::ns-between q3 q4)
        :topv-ns      (:dc::ns-between q4 q5)
        :direct-ns    (:dc::ns-between d0 d1)
        :query-count  (:wat::core::length q)
        :into-count   (:wat::core::length pv)
        :direct-count (:wat::core::length direct)))))
