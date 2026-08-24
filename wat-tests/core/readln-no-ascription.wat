;; wat-tests/core/readln-no-ascription.wat — -> :T annihilation, the LAST arrow (readln).
;;
;; readln drops its `-> :T` result ascription (arc 258, Option A — the self-describing kill):
;; readln no longer forces the caller to ATTEST the type it is about to read; it reads what the
;; SELF-DESCRIBING EDN wire says (records-are-EDN, arc 234.7), exactly as recv'/select' do. The
;; decoded value's type is a fresh var pinned by the CONSUMER (mirror `infer_recv_prime`); a stray
;; `-> :T` in ascription position is now a located compile error.
;;
;; readln reads stdin, so it cannot be EXECUTED in a deftest' (no wire). This proves the CHECK-time
;; property instead: a bare `(readln)` in a consumer position (a foldl over (Vector :- [i64])) TYPE-CHECKS
;; — its element type is inferred as :i64 from the fold — inside an fn VALUE that is never applied,
;; so the read never fires. If bare-readln inference regressed, this file would fail to load.

(:wat::test::deftest :wat-tests::core::readln-no-ascription
  
  ;; `sum-stdin` is a well-typed fn (bare readln infers (Vector :- [i64]) from the foldl consumer);
  ;; it is bound and NEVER called, so stdin is untouched. The deftest' asserts a trivial truth —
  ;; the load-bearing proof is that the fn body type-checks with NO `-> :T` on readln.
  (:wat::core::let
    [sum-stdin
       (:wat::core::fn [] -> :wat::core::i64
         (:wat::core::foldl
           (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
             (:wat::core::i64::+ a b))
           0
           (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))]
    (:wat::test::assert-true true)))
