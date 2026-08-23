;; probe-118B-match-tco-drain.wat — stone 118.B lair probe #1 (THE TAIL VERSION)
;;
;; QUESTION: `stream->pvec` (wat/seq.wat:102) is the language's ONE Stream materializer, and its
;; doc claims "Tail-recursive (TCO trampoline keeps this O(1) Rust-stack regardless of stream
;; length)". Today it is written with `if` + the three-call walk (`empty?`/`first`/`rest`).
;; Stone B migrates it to `(match (next s) …)`. If `match` did NOT carry a tail position, that
;; migration would silently convert an O(1)-stack drain into an O(n)-stack one and SIGSEGV on
;; long streams (see tasks #58/#86: stack exhaustion here is a SILENT SIGSEGV, not a clean raise).
;;
;; `eval_match_tail` exists at src/runtime.rs:4560 and is dispatched at :4309 — but that is a
;; READ, and the 294 seam's standing alarm is DO NOT DESIGN THE STREAM TIER FROM READING.
;; This is the run.
;;
;; PASS = prints 200000 (the drain completed at a depth that the sibling control proves is
;; deep enough to detect a missing TCO). Its non-vacuity control is
;; `probe-118B-match-no-tco-control.wat` — same depth, same `match`, recursion moved OUT of tail
;; position; that one MUST die. A pass here with a passing control would prove nothing.

;; The MIGRATED shape stone B would give `stream->pvec`: one `next` per element (one force),
;; both halves bound by the match, tail-recursive in the Item arm.
(:wat::core::defn :probe::drain-next
  [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])
   s   <- (:wat::stream::Stream :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest)
      (:probe::drain-next (:wat::core::PersistentVector/conj acc value) rest))
    (:wat::stream::NextOutcome::Exhausted acc)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n      200000
     s      (:wat::core::map
              (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
              (:wat::core::range 0 n))
     out    (:probe::drain-next (:wat::core::PersistentVector) s)]
    (:wat::kernel::println (:wat::core::length out))))
