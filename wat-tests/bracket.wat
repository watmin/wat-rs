;; wat-tests/bracket.wat — native wat coverage of wat/bracket.wat (the brackets pool).
;;
;; Arc 259 S3.5a — the FIRST native-wat tests of the spawn surface, riding `deftest'`
;; (the pipe-model test macro on the new substrate). Brackets are thread-tier: each
;; `deftest'` spawns a test-body thread peer whose body spawns the bracket runners —
;; the dynamically-balanced pool running NESTED inside the test peer. This dogfoods
;; spawn-program' + recv' + brackets entirely in wat.
;;
;; The worker-id-vs-host-cpu-count distinct-set check stays a Rust probe
;; (probe_arc259_brackets_worker) — that is a substrate-boundary assertion (it reads
;; std::thread::available_parallelism as an independent oracle). These cover the wat
;; SURFACE: order-preserving map, side-effect each, and the per-runner map-worker.

;; ── map: doubles, in input order ─────────────────────────────────────────────
(:wat::test::deftest :wat-tests::bracket::map-doubles-in-order
  
  (:wat::test::assert-eq
    (:wat::bracket::map (:wat::spawn::thread)
      (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
    (:wat::core::Vector :- [:wat::core::i64] 2 4 6 8 10)))

;; ── map: 50 items, input order preserved despite dynamic balance ─────────────
(:wat::test::deftest :wat-tests::bracket::map-preserves-order-50
  
  (:wat::test::assert-eq
    (:wat::bracket::map (:wat::spawn::thread)
      (:wat::core::range 0 50)
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
    ;; Arc 118.2a — `map` is now lazy (Stream); assert-eq's param #2 needs a concrete
    ;; Vector here (this is ordinary test-body code, not a program-body macro, so the
    ;; wat-level `mapv` materializer from wat/seq.wat is reachable at runtime).
    (:wat::core::mapv
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2))
      (:wat::core::range 0 50))))

;; ── each: side-effect pool, returns nil (and drains all items) ────────────────
(:wat::test::deftest :wat-tests::bracket::each-returns-nil
  
  (:wat::test::assert-eq
    (:wat::bracket::each (:wat::spawn::thread)
      (:wat::core::range 0 10)
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
    nil))

;; ── map-worker: with a constant worker-init (ignoring the id), equals map ─────
;; Arc 170 gap J — map-worker absorbed `uses'`'s provisioning params; a plain caller passes
;; `nil` grant-handles, a no-op grant-fn/revoke-fn pair, and an EMPTY `(Vector :- [D])` (no Setup).
(:wat::test::deftest :wat-tests::bracket::map-worker-ignoring-wid-equals-map
  
  (:wat::test::assert-eq
    (:wat::bracket::map-worker (:wat::spawn::thread)
      (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
      (:wat::core::fn [_wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
      nil
      (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
      (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
      (:wat::core::Vector :- [:wat::core::nil]))
    (:wat::core::Vector :- [:wat::core::i64] 2 4 6)))

;; ── each-worker: per-runner side-effect pool, returns nil ─────────────────────
(:wat::test::deftest :wat-tests::bracket::each-worker-returns-nil
  
  (:wat::test::assert-eq
    (:wat::bracket::each-worker (:wat::spawn::thread)
      (:wat::core::range 0 5)
      (:wat::core::fn [_wid <- :wat::core::i64] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* x 2)))
      nil
      (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
      (:wat::core::fn [_g <- :wat::core::nil _pid <- :wat::core::i64] -> :wat::core::nil nil)
      (:wat::core::Vector :- [:wat::core::nil]))
    nil))
