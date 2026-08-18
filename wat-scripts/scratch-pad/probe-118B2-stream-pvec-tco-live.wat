;; probe-118B2-stream-pvec-tco-live.wat — STOP-1 verification against the ACTUAL migrated
;; `:wat::core::stream->pvec` (wat/seq.wat), not an isolated copy. `probe-118B-match-tco-drain.wat`
;; proved `match` carries a tail position at n=200,000 with a private clone of the shape; this
;; drains the SAME depth through the real stdlib entry point (`into (PersistentVector) ...`),
;; which is the actual call every `(into (PersistentVector) stream)` / rete.wat site makes.
;; PASS = prints 200000 without a SIGSEGV. Scratch, per CLAUDE.md.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n   200000
     s   (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) (:wat::core::range 0 n))
     out (:wat::core::into (:wat::core::PersistentVector) s)]
    (:wat::kernel::println (:wat::core::length out))))
