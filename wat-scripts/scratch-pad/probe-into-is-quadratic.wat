;; probe-into-is-quadratic.wat — arc 278. Is `(into [] <stream>)` O(n^2)?
;;
;; THE CLAIM UNDER TEST: `stream->vec` (wat/seq.wat:70) drains a Stream by calling `conj` once
;; per element, and `vector_conj_inner` (src/collection/eval.rs:542) does
;;     let mut out = (**xs).clone();  out.push(item);
;; — a FULL copy of the accumulator per element. That makes the language's standard materializer,
;; `(into [] (map f coll))`, quadratic. 101 call sites route through it.
;;
;; THE ISOLATION. Two paths, SAME n, SAME output vector:
;;   A  (into [] (map identity src))  — map yields a Stream -> stream->vec -> n x conj  [suspect]
;;   B  (into [] src)                 — Vector+Vector clause -> native `concat`, ONE shot  [control]
;; Same element count, same values, same result type. The ONLY difference is how the accumulator
;; is built. If A is superlinear while B stays flat, the accumulation is the cost and the closure
;; is not — which no amount of reading the code can establish.
;;
;; The identity closure is deliberate: it makes A's per-element WORK as close to zero as the
;; language allows, so anything A costs above B is accumulation, not computation.
;;
;; stdin  = [n]
;; stdout = one #probe/IntoCost EDN line.

(:wat::core::defrecord :probe::IntoCost
  [n <- :wat::core::i64
   stream-drain-ns <- :wat::core::i64      ;; A — via map -> Stream -> stream->vec (n x Vec-copy conj)
   native-concat-ns <- :wat::core::i64     ;; B — via the (Vector,Vector) clause -> one concat
   pvec-drain-ns <- :wat::core::i64        ;; C — same stream drain, rpds accumulator (structural sharing)
   drain-len <- :wat::core::i64            ;; non-vacuity: all three must equal n, or the comparison is void
   concat-len <- :wat::core::i64
   pvec-len <- :wat::core::i64])

(:wat::core::defn :iq::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; src n — a (Vector :- [i64]) of n elements, built ONCE and outside both timed regions so the
;; construction cost is charged to neither path.
(:wat::core::defn :iq::src [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::range 0 n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [params (:wat::core::match (:wat::kernel::readln )
              ((:wat::kernel::ReadlnOutcome::Datum __d) __d)
              (:wat::kernel::ReadlnOutcome::Eof     (:wat::kernel::assertion-failed! "readln: eof"  :wat::core::None :wat::core::None))
              (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop" :wat::core::None :wat::core::None)))
     n    (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [n]")
     src  (:iq::src n)

     ;; A — the suspect: map yields a lazy Stream, `into` drains it with n conj calls.
     a0   (:wat::time::now)
     va   (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
            (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) src))
     a1   (:wat::time::now)

     ;; B — the control: Vector into Vector hits the `concat` clause, one native build.
     b0   (:wat::time::now)
     vb   (:wat::core::into (:wat::core::Vector :- [:wat::core::i64]) src)
     b1   (:wat::time::now)

     ;; C — the SAME stream drain, but accumulating into a PersistentVector. `stream->pvec`
     ;; conj's an rpds VectorSync, whose push_back SHARES structure rather than copying the
     ;; whole buffer. If the quadratic is the Vec copy (and not the stream machinery), C is
     ;; linear and the O(n) drain already exists — no new Rust required.
     c0   (:wat::time::now)
     vc   (:wat::core::into (:wat::core::PersistentVector)
            (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) src))
     c1   (:wat::time::now)]

    (:wat::kernel::println
      (:probe::IntoCost
        :n n
        :stream-drain-ns  (:iq::ns-between a0 a1)
        :native-concat-ns (:iq::ns-between b0 b1)
        :pvec-drain-ns    (:iq::ns-between c0 c1)
        :drain-len  (:wat::core::length va)
        :concat-len (:wat::core::length vb)
        :pvec-len   (:wat::vector::length vc)))))
