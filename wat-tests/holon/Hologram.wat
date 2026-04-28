;; wat-tests/holon/Hologram.wat — tests for arc 074 slice 1.
;;
;; Hologram is a coordinate-cell store with cosine readout.
;; HolonAST → HolonAST. The user supplies pos: f64 in [0, 100];
;; substrate maps that to floor(sqrt(d)) cells. `get` walks the two
;; adjacent cells (or one, if pos is exactly on a boundary) and
;; cosine-matches each candidate against the probe; returns the val
;; of the highest-cosine entry whose cosine satisfies the user's
;; filter func.
;;
;;   new   :: i64 -> Hologram                   ; d as a positive int
;;   put   :: Hologram, f64, AST, AST -> ()    ; pos in [0, 100]
;;   get   :: Hologram, f64, AST, fn(f64)->bool -> Option<AST>
;;   len   :: Hologram -> i64

;; Filters are inline `:wat::core::lambda`s — defined functions can't
;; (yet) be referenced as values by their keyword path; the lambda
;; form is what binds to a `:fn(...)` type.

;; ─── new + len: empty store ───────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-new-empty
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 0)))

;; ─── put + len: count increments ──────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-put-increments-len
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── put idempotent at same key ───────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-put-idempotent-on-same-key
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k  :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k v1))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k v2))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── get hits self with permissive filter ─────────────────────────
;;
;; Self-cosine is 1.0; any reasonable filter accepts.

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((accept-near-one :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool)
        (:wat::core::> cos 0.95)))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 k accept-near-one))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── get returns None when filter rejects everything ──────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-rejects-via-filter
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((reject-all :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) false))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 k reject-all))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── get returns None on empty store ──────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-empty-returns-none
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 probe accept-any))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── get against a distant probe in the same cell ─────────────────
;;
;; Two unrelated atoms in the same cell — accept-any filter accepts
;; anything, so we get SOMETHING (the higher-cosine match wins).

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-distant-probe-still-returns
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :a-out))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k1 v1))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :unrelated))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 probe accept-any))
     ((is-some :bool)
      (:wat::core::match got -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq is-some true)))

;; ─── get does NOT find an entry in a distant cell ─────────────────
;;
;; pos=5 puts in cell 5; pos=80 looks in cells 80..81. Different
;; neighborhood, no candidates — None regardless of filter strictness.

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-distant-cell-misses
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 80.0 k accept-any))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── len counts entries across cells ──────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-len-across-cells
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
     ((_ :()) (:wat::holon::Hologram/put store  5.0 k1 v1))
     ((_ :()) (:wat::holon::Hologram/put store 80.0 k2 v2))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 2)))

;; ─── presence-floor and coincident-floor expose substrate values ──

(:wat::test::deftest :wat-tests::holon::Hologram::test-presence-floor-positive
  ()
  (:wat::core::let*
    (((floor :f64) (:wat::holon::presence-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-floor-positive
  ()
  (:wat::core::let*
    (((floor :f64) (:wat::holon::coincident-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))

;; ─── dim accessor returns the d the store was built with ─────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-dim-returns-construction-d
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((d :i64) (:wat::holon::Hologram/dim store)))
    (:wat::test::assert-eq d 10000)))

;; ─── coincident-get composes filter-coincident at the store's d ──
;;
;; Self-cosine = 1.0 → coincident-filter accepts → Some(stored val).
;; The user passes no filter and no d; the convenience reads both
;; off the store.

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 5.0 k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── present-get composes filter-present at the store's d ────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-present-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/present-get store 5.0 k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── coincident-get on empty store returns None ──────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-get-empty-none
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 5.0 probe))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ═══════════════════════════════════════════════════════════════════
;; Deep integration — populates a realistic store and exercises every
;; behavior the design claims, end-to-end. Each behavior is a
;; standalone test so the failure surface localizes; together they
;; constitute the "this works as we want" proof.
;; ═══════════════════════════════════════════════════════════════════

;; ─── Behavior 1: cell isolation ──────────────────────────────────
;;
;; Entries put at distant positions (cells 5 and 80 at d=10000) must
;; never bleed into each other. The strong claim is two-fold:
;;
;;   (a) coincident-get at pos=80 with alpha-key returns None — alpha
;;       is in cell 5, the spread at pos=80 covers cells 80/81 only,
;;       so the only candidate is omega (cosine(alpha,omega) ≈ 0,
;;       well below the coincident floor).
;;
;;   (b) an accept-any get at pos=80 returns the LOCAL cell's argmax
;;       (omega-val), NEVER the distant cell's content (alpha-val).
;;       The distinction matters: cell isolation is about WHAT GETS
;;       SCANNED, not about whether ANYTHING gets returned.

(:wat::test::deftest :wat-tests::holon::Hologram::integ-cell-isolation
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((alpha-key :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((alpha-val :wat::holon::HolonAST) (:wat::holon::leaf :alpha-val))
     ((omega-key :wat::holon::HolonAST) (:wat::holon::leaf :omega))
     ((omega-val :wat::holon::HolonAST) (:wat::holon::leaf :omega-val))
     ((_ :()) (:wat::holon::Hologram/put store  5.0 alpha-key alpha-val))
     ((_ :()) (:wat::holon::Hologram/put store 80.0 omega-key omega-val))
     ;; (a) Coincident-get at pos=80 with alpha-key — alpha not in
     ;; cells 80/81; omega IS in cell 80 but cosine(alpha,omega) is
     ;; nowhere near the coincident floor. Result: None.
     ((coin :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 80.0 alpha-key))
     ((coin-rejected :bool)
      (:wat::core::match coin -> :bool ((Some _) false) (:None true)))
     ;; (b) Accept-any at pos=80 with alpha-key — returns the LOCAL
     ;; argmax (omega-val), NOT alpha-val. Verifies the distant cell
     ;; (cell 5) was never scanned.
     ((any :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 80.0 alpha-key
        (:wat::holon::filter-accept-any)))
     ((any-returned-omega :bool)
      (:wat::core::match any -> :bool
        ((Some h) (:wat::core::= h omega-val))
        (:None    false))))
    ;; Both must hold to prove cell isolation: distant cell unreachable
    ;; via strict filter; permissive filter sees only local cell.
    (:wat::test::assert-eq
      (:wat::core::if coin-rejected -> :bool any-returned-omega false)
      true)))

;; ─── Behavior 2: cosine discrimination within a single cell ──────
;;
;; Two distinct keys in the SAME cell. A coincident-get with the
;; first key must return the FIRST key's val, not the sibling's.
;; Self-cosine = 1.0 wins over any cross-key cosine.

(:wat::test::deftest :wat-tests::holon::Hologram::integ-cosine-discrimination
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((rsi-thought :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::leaf :neutral)))
     ((rsi-val :wat::holon::HolonAST) (:wat::holon::leaf :rsi-val))
     ((macd-thought :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :macd-thought)
        (:wat::holon::leaf :neutral)))
     ((macd-val :wat::holon::HolonAST) (:wat::holon::leaf :macd-val))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 rsi-thought rsi-val))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 macd-thought macd-val))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 5.0 rsi-thought))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found rsi-val)))

;; ─── Behavior 3: cell-spread at boundaries ───────────────────────
;;
;; pos=2.0 puts at cell 2 (floor=2, ceil=2). pos=2.5 queries cells 2
;; AND 3 (floor=2, ceil=3). The entry stored at pos=2.0 must be
;; findable from a probe at pos=2.5 — the spread is what makes
;; coordinate-cell + cosine-readout work for non-aligned queries.

(:wat::test::deftest :wat-tests::holon::Hologram::integ-spread-finds-adjacent-cell
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((_ :()) (:wat::holon::Hologram/put store 2.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 2.5 k))
     ((is-some :bool)
      (:wat::core::match got -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq is-some true)))

;; ─── Behavior 4: filter strictness divergence ────────────────────
;;
;; coincident-get rejects what filter-accept-any accepts. Same
;; store, same pos, same probe — different filter, different result.

(:wat::test::deftest :wat-tests::holon::Hologram::integ-strictness-divergence
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((stored-key :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((stored-val :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 stored-key stored-val))
     ((unrelated :wat::holon::HolonAST) (:wat::holon::leaf :unrelated))
     ;; Strict: rejects unrelated probe (cosine ~0).
     ((coin :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 5.0 unrelated))
     ((coin-rejected :bool)
      (:wat::core::match coin -> :bool
        ((Some _) false)
        (:None    true)))
     ;; Permissive: accepts the population's argmax regardless.
     ((any :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 unrelated
        (:wat::holon::filter-accept-any)))
     ((any-accepted :bool)
      (:wat::core::match any -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq
      (:wat::core::if coin-rejected -> :bool any-accepted false)
      true)))

;; ─── Behavior 5: Thermometer-bearing forms preserve identity ─────
;;
;; A Bind(Atom, Thermometer) — the trader's thought shape — must
;; round-trip through put/get without losing fidelity. Self-cosine
;; on a Thermometer-bearing form is 1.0 just like any other form;
;; coincident-get accepts the round-trip.

(:wat::test::deftest :wat-tests::holon::Hologram::integ-thermometer-roundtrip
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((rsi-at-70 :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-thought)
        (:wat::holon::Thermometer 70.0 0.0 100.0)))
     ((next-form :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::leaf :rsi-up-pressure)
        (:wat::holon::Thermometer 0.7 -1.0 1.0)))
     ((_ :()) (:wat::holon::Hologram/put store 30.0 rsi-at-70 next-form))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 30.0 rsi-at-70))
     ((retrieved :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable-thermometer)))))
    (:wat::test::assert-eq retrieved next-form)))

;; ─── Behavior 6: population scale — 5 distinct entries scattered
;;
;; Each (k_n, v_n) round-trips its key to the matching val. The
;; store handles non-trivial population without keys interfering.
;; Tests both put-many-cells and get-from-each.

(:wat::test::deftest :wat-tests::holon::Hologram::integ-population-roundtrip
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k0 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v0 :wat::holon::HolonAST) (:wat::holon::leaf :alpha-result))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :beta-result))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma-result))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :delta))
     ((v3 :wat::holon::HolonAST) (:wat::holon::leaf :delta-result))
     ((k4 :wat::holon::HolonAST) (:wat::holon::leaf :epsilon))
     ((v4 :wat::holon::HolonAST) (:wat::holon::leaf :epsilon-result))
     ((_ :()) (:wat::holon::Hologram/put store  0.5 k0 v0))
     ((_ :()) (:wat::holon::Hologram/put store 25.0 k1 v1))
     ((_ :()) (:wat::holon::Hologram/put store 50.0 k2 v2))
     ((_ :()) (:wat::holon::Hologram/put store 75.0 k3 v3))
     ((_ :()) (:wat::holon::Hologram/put store 99.0 k4 v4))
     ((g0 :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store  0.5 k0))
     ((g1 :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 25.0 k1))
     ((g2 :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 50.0 k2))
     ((g3 :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 75.0 k3))
     ((g4 :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 99.0 k4))
     ((sentinel :wat::holon::HolonAST) (:wat::holon::leaf :unreachable-pop))
     ((r0 :wat::holon::HolonAST)
      (:wat::core::match g0 -> :wat::holon::HolonAST ((Some h) h) (:None sentinel)))
     ((r1 :wat::holon::HolonAST)
      (:wat::core::match g1 -> :wat::holon::HolonAST ((Some h) h) (:None sentinel)))
     ((r2 :wat::holon::HolonAST)
      (:wat::core::match g2 -> :wat::holon::HolonAST ((Some h) h) (:None sentinel)))
     ((r3 :wat::holon::HolonAST)
      (:wat::core::match g3 -> :wat::holon::HolonAST ((Some h) h) (:None sentinel)))
     ((r4 :wat::holon::HolonAST)
      (:wat::core::match g4 -> :wat::holon::HolonAST ((Some h) h) (:None sentinel)))
     ((all-match :bool)
      (:wat::core::if (:wat::core::= r0 v0) -> :bool
        (:wat::core::if (:wat::core::= r1 v1) -> :bool
          (:wat::core::if (:wat::core::= r2 v2) -> :bool
            (:wat::core::if (:wat::core::= r3 v3) -> :bool
              (:wat::core::= r4 v4)
              false)
            false)
          false)
        false))
     ((total :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq
      (:wat::core::if all-match -> :bool
        (:wat::core::if (:wat::core::= total 5) -> :bool true false)
        false)
      true)))
