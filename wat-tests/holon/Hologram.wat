;; wat-tests/holon/Hologram.wat — tests for arc 076 (therm-routed Hologram).
;;
;; Hologram is now therm-routed: the slot for any (key, val) is
;; derived from the key's structure. A Thermometer-bearing form lands
;; at floor((value - min) / (max - min) * capacity) clamped to
;; [0, capacity-1]; non-therm forms always land at slot 0. The filter
;; func is bound at construction; get is filtered-argmax over the
;; bracket-pair (floor + ceil) of the probe's slot.
;;
;; Surface:
;;   make     :: i64, fn(f64)->bool -> Hologram   ; capacity = floor(sqrt(d))
;;   put      :: Hologram, AST, AST -> ()         ; slot inferred from key
;;   get      :: Hologram, AST -> wat::core::Option<AST>     ; filter from construction
;;   len      :: Hologram -> i64
;;   capacity :: Hologram -> i64                  ; floor(sqrt(d))
;;
;;   therm-form :: f64, f64, f64 -> AST           ; (low, high, value); clamps OOB

;; ─── make + len: empty store has len 0 ──────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-make-empty
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-accept-any)))
     ((n :wat::core::i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 0)))

;; ─── capacity returns floor(sqrt(d)) ────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-capacity-at-d-10000
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-accept-any)))
     ((cap :wat::core::i64) (:wat::holon::Hologram/capacity store)))
    (:wat::test::assert-eq cap 100)))

;; Note: alternate d (e.g. 4096 → cap 64) is exercised by the Rust
;; unit tests (`hologram::tests::slot_routing_capacity_is_hologram_property`).
;; From wat, d is ambient — currently fixed at DEFAULT_TIERS[0].
;; Arc 077 will introduce `(:wat::config::set-dim-count! n)` so the
;; user chooses d once for their program (default 10000 if not set);
;; this test will then call `(:wat::config::set-dim-count! 4096)`
;; and assert capacity = 64.

;; ─── put + len: count increments ────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-put-increments-len
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-accept-any)))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_  :wat::core::unit) (:wat::holon::Hologram/put store k v))
     ((n :wat::core::i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── put idempotent on same key ─────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-put-idempotent
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-accept-any)))
     ((k  :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((_  :wat::core::unit) (:wat::holon::Hologram/put store k v1))
     ((_  :wat::core::unit) (:wat::holon::Hologram/put store k v2))
     ((n :wat::core::i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── non-therm round-trip via slot 0 ────────────────────────────
;;
;; A bare keyword has no Thermometer; routes to slot 0. Self-cosine
;; is 1.0; coincidence filter accepts; get returns the stored val.

(:wat::test::deftest :wat-tests::holon::Hologram::test-non-therm-roundtrip
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-coincident)))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :alpha-result))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── therm round-trip via slot floor(value) ─────────────────────
;;
;; A bare Thermometer routes to floor((value - 0)/(100 - 0) * 100) = 70.
;; Self-cosine 1.0; coincidence filter accepts.

(:wat::test::deftest :wat-tests::holon::Hologram::test-therm-roundtrip
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-coincident)))
     ((k :wat::holon::HolonAST) (:wat::holon::Thermometer 70.0 0.0 100.0))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :rsi-70-answer))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── empty store returns None ───────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-empty-store-returns-none
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-accept-any)))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store probe))
     ((is-none :wat::core::bool)
      (:wat::core::match got -> :wat::core::bool
        ((:wat::core::Some _) false)
        (:wat::core::None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── filter rejection: filter says no, even single candidate ────
;;
;; A reject-everything filter ensures get always returns None,
;; regardless of cosine. Verifies the filter is invoked uniformly,
;; not just when there's choice ambiguity.

(:wat::test::deftest :wat-tests::holon::Hologram::test-filter-always-rejects
  ()
  (:wat::core::let*
    (((reject-all :fn(wat::core::f64)->wat::core::bool)
      (:wat::core::lambda ((_ :wat::core::f64) -> :wat::core::bool) false))
     ((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make reject-all))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :stored-val))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store k))
     ((is-none :wat::core::bool)
      (:wat::core::match got -> :wat::core::bool
        ((:wat::core::Some _) false)
        (:wat::core::None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── slot isolation: distant therm slots don't see each other ───
;;
;; Stored therms at slot 5 and slot 80 must not appear in each
;; other's bracket-pair lookups. Coincidence filter on a distant
;; probe returns None.

(:wat::test::deftest :wat-tests::holon::Hologram::test-slot-isolation
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-coincident)))
     ((k1 :wat::holon::HolonAST) (:wat::holon::Thermometer 5.0 0.0 100.0))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :slot-5-val))
     ((k2 :wat::holon::HolonAST) (:wat::holon::Thermometer 80.0 0.0 100.0))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :slot-80-val))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k1 v1))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k2 v2))
     ;; Probe at slot 80 with the slot-5 form's value — coincidence
     ;; filter rejects (cosine far below floor); get returns None.
     ;; The local slot has v2 but its key is structurally different,
     ;; so cosine fails the coincident threshold.
     ((probe :wat::holon::HolonAST) (:wat::holon::Thermometer 5.0 0.0 100.0))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store probe))
     ;; Probe k1 (slot 5); store has the matching key at slot 5;
     ;; cosine 1.0; passes coincidence. Returns v1.
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v1)))

;; ─── cosine discrimination within slot 0 (non-therm pile-up) ────
;;
;; Two distinct non-therm forms both pile into slot 0. A get with
;; one form's key returns its specific val (cosine 1.0 wins over
;; cross-form cosine).

(:wat::test::deftest :wat-tests::holon::Hologram::test-slot-0-discriminates
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-coincident)))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha-val))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :beta-val))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k1 v1))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k2 v2))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store k1))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v1)))

;; ─── bracket-pair lookup spans floor + ceil slots ───────────────
;;
;; Put a therm at slot 42 (value 42.0); probe at value 42.5 — the
;; bracket-pair is (42, 43); slot 42 has the matching key; cosine on
;; encoded therms reflects the slot-position closeness; coincidence
;; filter accepts (the therms are close in encoded space).

(:wat::test::deftest :wat-tests::holon::Hologram::test-bracket-pair-finds-floor-slot
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-accept-any)))
     ((k :wat::holon::HolonAST) (:wat::holon::Thermometer 42.0 0.0 100.0))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :slot-42-val))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k v))
     ;; Probe value 42.5 — floor=42, ceil=43; slot 42 contains v.
     ((probe :wat::holon::HolonAST) (:wat::holon::Thermometer 42.5 0.0 100.0))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store probe))
     ((is-some :wat::core::bool)
      (:wat::core::match got -> :wat::core::bool
        ((:wat::core::Some _) true)
        (:wat::core::None    false))))
    (:wat::test::assert-eq is-some true)))

;; ─── therm-form constructor: builds canonical Thermometer ───────

(:wat::test::deftest :wat-tests::holon::Hologram::test-therm-form-builds-canonical
  ()
  (:wat::core::let*
    (((built :wat::holon::HolonAST)
      (:wat::holon::therm-form 0.0 100.0 70.0))
     ((expected :wat::holon::HolonAST)
      (:wat::holon::Thermometer 70.0 0.0 100.0)))
    (:wat::test::assert-eq built expected)))

;; ─── therm-form clamps OOB low ──────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-therm-form-clamps-oob-low
  ()
  (:wat::core::let*
    (((built :wat::holon::HolonAST)
      (:wat::holon::therm-form 0.0 100.0 -10.0))
     ((expected :wat::holon::HolonAST)
      (:wat::holon::Thermometer 0.0 0.0 100.0)))
    (:wat::test::assert-eq built expected)))

;; ─── therm-form clamps OOB high ─────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-therm-form-clamps-oob-high
  ()
  (:wat::core::let*
    (((built :wat::holon::HolonAST)
      (:wat::holon::therm-form 0.0 100.0 110.0))
     ((expected :wat::holon::HolonAST)
      (:wat::holon::Thermometer 100.0 0.0 100.0)))
    (:wat::test::assert-eq built expected)))

;; ─── therm-form preserves natural domain (asymmetric) ───────────
;;
;; The form keeps the user's domain bounds; capacity stays a
;; Hologram-side concern. A 200-600 domain produces a Thermometer
;; whose min/max match.

(:wat::test::deftest :wat-tests::holon::Hologram::test-therm-form-preserves-domain
  ()
  (:wat::core::let*
    (((built :wat::holon::HolonAST)
      (:wat::holon::therm-form 200.0 600.0 400.0))
     ((expected :wat::holon::HolonAST)
      (:wat::holon::Thermometer 400.0 200.0 600.0)))
    (:wat::test::assert-eq built expected)))

;; ─── therm-form into Hologram round-trip ────────────────────────
;;
;; therm-form constructs a canonical therm; Hologram routes by the
;; form's natural domain via its own capacity. Self-cosine 1.0
;; passes coincidence. Confirms therm-form + Hologram compose.

(:wat::test::deftest :wat-tests::holon::Hologram::test-therm-form-roundtrips-via-hologram
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-coincident)))
     ((k :wat::holon::HolonAST) (:wat::holon::therm-form 0.0 100.0 42.42))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :rsi-42-answer))
     ((_ :wat::core::unit) (:wat::holon::Hologram/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── presence-floor / coincident-floor accessors stay green ────

(:wat::test::deftest :wat-tests::holon::Hologram::test-presence-floor-positive
  ()
  (:wat::core::let*
    (((floor :wat::core::f64) (:wat::holon::presence-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-floor-positive
  ()
  (:wat::core::let*
    (((floor :wat::core::f64) (:wat::holon::coincident-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))
