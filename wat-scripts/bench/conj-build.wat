;; wat-scripts/bench/conj-build.wat — conj-build perf: Vector and List, N elements, by `conj`.
;;
;; Arc 2026-10 persistent-vector-and-list. Builds a Vector of N i64 by repeated
;; `:wat::vec::conj` (APPEND) and a List of N i64 by repeated `:wat::linkedlist::conj`
;; (PREPEND), each timed separately, and prints a checksum (sum of elements) for each
;; so a correctness mismatch between the pre-swap and post-swap runtime would show up
;; as a different number, not just a different time.
;;
;; Usage (stdin = an EDN i64 N; stdout = an EDN :perf::ConjResult map):
;;   echo '10000' | cargo wat ./wat-scripts/bench/conj-build.wat

;; Two separate records, printed as soon as each side finishes building+summing — so a
;; run where ONE side (typically the pre-swap List, O(n^2)) is slow still shows the OTHER
;; side's line on stdout before it times out, instead of both being silently lost because
;; a single combined record waits on the slower half.
(:wat::core::defrecord :perf::VecResult
  [n            <- :wat::core::i64
   vec-checksum <- :wat::core::i64
   vec-ns       <- :wat::core::i64])

(:wat::core::defrecord :perf::ListResult
  [n             <- :wat::core::i64
   list-checksum <- :wat::core::i64
   list-ns       <- :wat::core::i64])

(:wat::core::defn :perf::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

;; build-vec n — a Vector of [0, 1, .. n-1] built by n persistent `conj` (APPEND).
(:wat::core::defn :perf::build-vec [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::i64])  i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::i64])
      (:wat::vec::conj acc i))
    (:wat::core::Vector :- [:wat::core::i64])
    (:wat::core::range 0 n)))

;; build-list n — a List of [n-1, .. 1, 0] built by n persistent `conj` (PREPEND).
(:wat::core::defn :perf::build-list [n <- :wat::core::i64] -> (:wat::core::List :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::List :- [:wat::core::i64])  i <- :wat::core::i64] -> (:wat::core::List :- [:wat::core::i64])
      (:wat::linkedlist::conj acc i))
    (:wat::core::List)
    (:wat::core::range 0 n)))

;; sum-vec v — fold `+` over a Vector's elements.
(:wat::core::defn :perf::sum-vec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ acc x))
    0
    v))

;; sum-list l — fold `+` over a List's elements.
(:wat::core::defn :perf::sum-list [l <- (:wat::core::List :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ acc x))
    0
    l))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [n        (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])
                    v0       (:wat::time::now)
                    the-vec  (:perf::build-vec n)
                    v1       (:wat::time::now)
                    vec-sum  (:perf::sum-vec the-vec)
                    vec-ns   (:perf::ns-between v0 v1)
                    _        (:wat::kernel::println (:perf::VecResult :n n :vec-checksum vec-sum :vec-ns vec-ns))
                    l0       (:wat::time::now)
                    the-list (:perf::build-list n)
                    l1       (:wat::time::now)
                    list-sum (:perf::sum-list the-list)
                    list-ns  (:perf::ns-between l0 l1)]
    (:wat::kernel::println (:perf::ListResult :n n :list-checksum list-sum :list-ns list-ns))))
