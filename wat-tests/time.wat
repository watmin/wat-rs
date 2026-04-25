;; wat-tests/time.wat — :wat::time::* surface tests (arc 056).
;;
;; Twelve deftests covering the 9 primitives + composition scenarios.
;; Direct `(:wat::test::deftest)` calls with empty prelude — the
;; primitives live at runtime-dispatch level (no `(load!)` needed).
;;
;; Coverage:
;;   - now returns an Instant in a sane epoch range
;;   - at, at-millis, at-nanos round-trips
;;   - from-iso8601 / to-iso8601 round-trips at digits 0/3/9
;;   - parse failure returns :None
;;   - to-iso8601 clamps digits outside [0, 9]
;;   - elapsed-via-subtract using two `now` calls
;;   - epoch accessor consistency


;; ─── now ──────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::time::test-now-returns-instant
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::now))
     ((s :i64) (:wat::time::epoch-seconds i)))
    ;; Sanity: now is post-2020 (> 1577836800 = 2020-01-01T00:00:00Z).
    ;; This file's author won't see year-2200 problems.
    (:wat::test::assert-eq (:wat::core::> s 1577836800) true)))


;; ─── at — epoch construction ──────────────────────────────────────

(:wat::test::deftest :wat-tests::time::test-at-zero-is-epoch
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ((s :String) (:wat::time::to-iso8601 i 0)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00Z")))


;; ─── at-millis / at — equivalence at second boundary ──────────────

(:wat::test::deftest :wat-tests::time::test-at-millis-matches-at
  ()
  (:wat::core::let*
    (((a :wat::time::Instant) (:wat::time::at 1))
     ((b :wat::time::Instant) (:wat::time::at-millis 1000))
     ((sa :i64) (:wat::time::epoch-seconds a))
     ((sb :i64) (:wat::time::epoch-seconds b)))
    (:wat::test::assert-eq sa sb)))


;; ─── at-nanos / at-millis — equivalence at ms boundary ────────────

(:wat::test::deftest :wat-tests::time::test-at-nanos-matches-at-millis
  ()
  (:wat::core::let*
    (((a :wat::time::Instant) (:wat::time::at-millis 1000))
     ((b :wat::time::Instant) (:wat::time::at-nanos 1000000000))
     ((ma :i64) (:wat::time::epoch-millis a))
     ((mb :i64) (:wat::time::epoch-millis b)))
    (:wat::test::assert-eq ma mb)))


;; ─── from-iso8601 / to-iso8601 — round-trip 3 digits ──────────────

(:wat::test::deftest :wat-tests::time::test-iso8601-roundtrip-3-digits
  ()
  (:wat::core::let*
    (((parsed :Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "2026-04-25T14:30:42.123Z")))
    (:wat::core::match parsed -> :()
      ((Some i)
        (:wat::core::let*
          (((s :String) (:wat::time::to-iso8601 i 3)))
          (:wat::test::assert-eq s "2026-04-25T14:30:42.123Z")))
      (:None
        (:wat::kernel::assertion-failed!
          "from-iso8601 returned None for valid input" :None :None)))))


;; ─── from-iso8601 / to-iso8601 — round-trip 9 digits ──────────────

(:wat::test::deftest :wat-tests::time::test-iso8601-roundtrip-9-digits
  ()
  (:wat::core::let*
    (((parsed :Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "2026-04-25T14:30:42.123456789Z")))
    (:wat::core::match parsed -> :()
      ((Some i)
        (:wat::core::let*
          (((s :String) (:wat::time::to-iso8601 i 9)))
          (:wat::test::assert-eq s "2026-04-25T14:30:42.123456789Z")))
      (:None
        (:wat::kernel::assertion-failed!
          "from-iso8601 returned None for nanosecond-precision input"
          :None :None)))))


;; ─── from-iso8601 — :None on parse failure ────────────────────────

(:wat::test::deftest :wat-tests::time::test-iso8601-parse-failure-is-none
  ()
  (:wat::core::let*
    (((parsed :Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "not-a-real-iso-string"))
     ((is-none? :bool)
      (:wat::core::match parsed -> :bool
        ((Some _) false)
        (:None true))))
    (:wat::test::assert-eq is-none? true)))


;; ─── to-iso8601 — digits = 0 produces no fractional portion ───────

(:wat::test::deftest :wat-tests::time::test-to-iso8601-digits-zero
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at-millis 1234567890123))
     ((s :String) (:wat::time::to-iso8601 i 0)))
    (:wat::test::assert-eq s "2009-02-13T23:31:30Z")))


;; ─── to-iso8601 — digits clamp to 9 from above ────────────────────

(:wat::test::deftest :wat-tests::time::test-to-iso8601-clamps-digits-high
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ;; digits = 42 should clamp to 9 — 9 zeroes for the epoch.
     ((s :String) (:wat::time::to-iso8601 i 42)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00.000000000Z")))


;; ─── to-iso8601 — digits clamp to 0 from below ────────────────────

(:wat::test::deftest :wat-tests::time::test-to-iso8601-clamps-digits-low
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ;; digits = -5 should clamp to 0 — no fractional portion.
     ((s :String) (:wat::time::to-iso8601 i -5)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00Z")))


;; ─── Duration measurement — two `now` calls + integer subtract ────

(:wat::test::deftest :wat-tests::time::test-elapsed-via-subtract
  ()
  (:wat::core::let*
    (((start :wat::time::Instant) (:wat::time::now))
     ((end :wat::time::Instant) (:wat::time::now))
     ((s-start :i64) (:wat::time::epoch-millis start))
     ((s-end :i64) (:wat::time::epoch-millis end))
     ((delta :i64) (:wat::core::- s-end s-start)))
    ;; Two `now` calls in immediate succession produce delta >= 0.
    ;; (NTP could move clock backwards; for the test environment,
    ;; same-process same-second calls reliably observe non-negative
    ;; delta. Documented risk — see DESIGN Q1.)
    (:wat::test::assert-eq (:wat::core::>= delta 0) true)))


;; ─── epoch-millis is epoch-seconds * 1000 + sub-second portion ────

(:wat::test::deftest :wat-tests::time::test-epoch-accessors-consistent
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at-millis 1234567890123))
     ((sec :i64) (:wat::time::epoch-seconds i))
     ((ms :i64) (:wat::time::epoch-millis i))
     ((derived :i64) (:wat::core::* sec 1000)))
    ;; ms truncates to int, sec*1000 = 1234567890000, ms = 1234567890123.
    ;; Difference is the sub-second portion (123 ms).
    (:wat::test::assert-eq (:wat::core::- ms derived) 123)))
