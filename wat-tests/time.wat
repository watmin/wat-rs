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
     ((s :wat::core::i64) (:wat::time::epoch-seconds i)))
    ;; Sanity: now is post-2020 (> 1577836800 = 2020-01-01T00:00:00Z).
    ;; This file's author won't see year-2200 problems.
    (:wat::test::assert-eq (:wat::core::> s 1577836800) true)))


;; ─── at — epoch construction ──────────────────────────────────────

(:wat::test::deftest :wat-tests::time::test-at-zero-is-epoch
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ((s :wat::core::String) (:wat::time::to-iso8601 i 0)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00Z")))


;; ─── at-millis / at — equivalence at second boundary ──────────────

(:wat::test::deftest :wat-tests::time::test-at-millis-matches-at
  ()
  (:wat::core::let*
    (((a :wat::time::Instant) (:wat::time::at 1))
     ((b :wat::time::Instant) (:wat::time::at-millis 1000))
     ((sa :wat::core::i64) (:wat::time::epoch-seconds a))
     ((sb :wat::core::i64) (:wat::time::epoch-seconds b)))
    (:wat::test::assert-eq sa sb)))


;; ─── at-nanos / at-millis — equivalence at ms boundary ────────────

(:wat::test::deftest :wat-tests::time::test-at-nanos-matches-at-millis
  ()
  (:wat::core::let*
    (((a :wat::time::Instant) (:wat::time::at-millis 1000))
     ((b :wat::time::Instant) (:wat::time::at-nanos 1000000000))
     ((ma :wat::core::i64) (:wat::time::epoch-millis a))
     ((mb :wat::core::i64) (:wat::time::epoch-millis b)))
    (:wat::test::assert-eq ma mb)))


;; ─── from-iso8601 / to-iso8601 — round-trip 3 digits ──────────────

(:wat::test::deftest :wat-tests::time::test-iso8601-roundtrip-3-digits
  ()
  (:wat::core::let*
    (((parsed :wat::core::Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "2026-04-25T14:30:42.123Z")))
    (:wat::core::match parsed -> :wat::core::unit
      ((Some i)
        (:wat::core::let*
          (((s :wat::core::String) (:wat::time::to-iso8601 i 3)))
          (:wat::test::assert-eq s "2026-04-25T14:30:42.123Z")))
      (:None
        (:wat::kernel::assertion-failed!
          "from-iso8601 returned None for valid input" :None :None)))))


;; ─── from-iso8601 / to-iso8601 — round-trip 9 digits ──────────────

(:wat::test::deftest :wat-tests::time::test-iso8601-roundtrip-9-digits
  ()
  (:wat::core::let*
    (((parsed :wat::core::Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "2026-04-25T14:30:42.123456789Z")))
    (:wat::core::match parsed -> :wat::core::unit
      ((Some i)
        (:wat::core::let*
          (((s :wat::core::String) (:wat::time::to-iso8601 i 9)))
          (:wat::test::assert-eq s "2026-04-25T14:30:42.123456789Z")))
      (:None
        (:wat::kernel::assertion-failed!
          "from-iso8601 returned None for nanosecond-precision input"
          :None :None)))))


;; ─── from-iso8601 — :None on parse failure ────────────────────────

(:wat::test::deftest :wat-tests::time::test-iso8601-parse-failure-is-none
  ()
  (:wat::core::let*
    (((parsed :wat::core::Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "not-a-real-iso-string"))
     ((is-none? :wat::core::bool)
      (:wat::core::match parsed -> :wat::core::bool
        ((Some _) false)
        (:None true))))
    (:wat::test::assert-eq is-none? true)))


;; ─── to-iso8601 — digits = 0 produces no fractional portion ───────

(:wat::test::deftest :wat-tests::time::test-to-iso8601-digits-zero
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at-millis 1234567890123))
     ((s :wat::core::String) (:wat::time::to-iso8601 i 0)))
    (:wat::test::assert-eq s "2009-02-13T23:31:30Z")))


;; ─── to-iso8601 — digits clamp to 9 from above ────────────────────

(:wat::test::deftest :wat-tests::time::test-to-iso8601-clamps-digits-high
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ;; digits = 42 should clamp to 9 — 9 zeroes for the epoch.
     ((s :wat::core::String) (:wat::time::to-iso8601 i 42)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00.000000000Z")))


;; ─── to-iso8601 — digits clamp to 0 from below ────────────────────

(:wat::test::deftest :wat-tests::time::test-to-iso8601-clamps-digits-low
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ;; digits = -5 should clamp to 0 — no fractional portion.
     ((s :wat::core::String) (:wat::time::to-iso8601 i -5)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00Z")))


;; ─── Duration measurement — two `now` calls + integer subtract ────

(:wat::test::deftest :wat-tests::time::test-elapsed-via-subtract
  ()
  (:wat::core::let*
    (((start :wat::time::Instant) (:wat::time::now))
     ((end :wat::time::Instant) (:wat::time::now))
     ((s-start :wat::core::i64) (:wat::time::epoch-millis start))
     ((s-end :wat::core::i64) (:wat::time::epoch-millis end))
     ((delta :wat::core::i64) (:wat::core::- s-end s-start)))
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
     ((sec :wat::core::i64) (:wat::time::epoch-seconds i))
     ((ms :wat::core::i64) (:wat::time::epoch-millis i))
     ((derived :wat::core::i64) (:wat::core::* sec 1000)))
    ;; ms truncates to int, sec*1000 = 1234567890000, ms = 1234567890123.
    ;; Difference is the sub-second portion (123 ms).
    (:wat::test::assert-eq (:wat::core::- ms derived) 123)))


;; ─── Arc 097 — Duration constructors ────────────────────────────────
;;
;; Seven unit constructors at :wat::time::* (Nanosecond, Microsecond,
;; Millisecond, Second, Minute, Hour, Day). Each takes :wat::core::i64, returns
;; a :wat::time::Duration carrying the equivalent nanos count.

(:wat::test::deftest :wat-tests::time::test-duration-nanosecond
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Nanosecond 42)))
    ;; Sanity: round-trip via render. 42 ns is the input.
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 42ns>")))

(:wat::test::deftest :wat-tests::time::test-duration-microsecond
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Microsecond 1)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 1000ns>")))

(:wat::test::deftest :wat-tests::time::test-duration-millisecond
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Millisecond 1)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 1000000ns>")))

(:wat::test::deftest :wat-tests::time::test-duration-second
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Second 1)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 1000000000ns>")))

(:wat::test::deftest :wat-tests::time::test-duration-minute
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Minute 1)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 60000000000ns>")))

(:wat::test::deftest :wat-tests::time::test-duration-hour
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Hour 1)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 3600000000000ns>")))

(:wat::test::deftest :wat-tests::time::test-duration-day
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Day 1)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 86400000000000ns>")))

;; Compositional sanity — arithmetic relationships.
(:wat::test::deftest :wat-tests::time::test-duration-hour-equals-60-minutes
  ()
  (:wat::core::let*
    (((h :wat::time::Duration) (:wat::time::Hour 1))
     ((m60 :wat::time::Duration) (:wat::time::Minute 60)))
    ;; Same nanos count → same Duration → same render.
    (:wat::test::assert-eq (:wat::core::show h) (:wat::core::show m60))))

(:wat::test::deftest :wat-tests::time::test-duration-day-equals-24-hours
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Day 1))
     ((h24 :wat::time::Duration) (:wat::time::Hour 24)))
    (:wat::test::assert-eq (:wat::core::show d) (:wat::core::show h24))))

;; Zero is a valid non-negative Duration.
(:wat::test::deftest :wat-tests::time::test-duration-zero-is-valid
  ()
  (:wat::core::let*
    (((d :wat::time::Duration) (:wat::time::Hour 0)))
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 0ns>")))


;; ─── Arc 097 slice 2 — Instant ± Duration arithmetic ────────────────
;;
;; :wat::time::- and :wat::time::+ dispatch on RHS variant:
;;   Instant - Duration -> Instant   (subtract interval)
;;   Instant - Instant  -> Duration  (elapsed between, panics on negative)
;;   Instant + Duration -> Instant   (advance by interval)
;;
;; ActiveSupport-shaped: same operator, different result types per the
;; RHS Value::Variant tag. The runtime checks at call time; the type
;; checker checks at expansion.

(:wat::test::deftest :wat-tests::time::test-instant-sub-duration-yields-instant
  ()
  (:wat::core::let*
    (((origin :wat::time::Instant) (:wat::time::at 1000000))
     ((one-min :wat::time::Duration) (:wat::time::Minute 1))
     ((earlier :wat::time::Instant) (:wat::time::- origin one-min))
     ((delta :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds origin)
        (:wat::time::epoch-seconds earlier))))
    ;; 1 minute earlier = 60 seconds back.
    (:wat::test::assert-eq delta 60)))

(:wat::test::deftest :wat-tests::time::test-instant-add-duration-yields-instant
  ()
  (:wat::core::let*
    (((origin :wat::time::Instant) (:wat::time::at 1000000))
     ((one-hour :wat::time::Duration) (:wat::time::Hour 1))
     ((later :wat::time::Instant) (:wat::time::+ origin one-hour))
     ((delta :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds later)
        (:wat::time::epoch-seconds origin))))
    ;; 1 hour later = 3600 seconds forward.
    (:wat::test::assert-eq delta 3600)))

(:wat::test::deftest :wat-tests::time::test-instant-sub-instant-yields-duration
  ()
  (:wat::core::let*
    (((later :wat::time::Instant) (:wat::time::at 1000060))
     ((earlier :wat::time::Instant) (:wat::time::at 1000000))
     ;; Instant - Instant -> Duration. RHS dispatch picks the right arm.
     ((elapsed :wat::time::Duration) (:wat::time::- later earlier)))
    ;; 60 seconds = 60_000_000_000 nanos.
    (:wat::test::assert-eq (:wat::core::show elapsed)
                           "<Duration 60000000000ns>")))

(:wat::test::deftest :wat-tests::time::test-add-then-sub-roundtrips
  ()
  (:wat::core::let*
    (((origin :wat::time::Instant) (:wat::time::at 1000000))
     ((d :wat::time::Duration) (:wat::time::Day 1))
     ((later :wat::time::Instant) (:wat::time::+ origin d))
     ((back :wat::time::Instant) (:wat::time::- later d))
     ((delta :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds back)
        (:wat::time::epoch-seconds origin))))
    ;; +1 day then -1 day returns to origin.
    (:wat::test::assert-eq delta 0)))

(:wat::test::deftest :wat-tests::time::test-zero-duration-is-identity-for-add
  ()
  (:wat::core::let*
    (((origin :wat::time::Instant) (:wat::time::at 1000000))
     ((zero :wat::time::Duration) (:wat::time::Hour 0))
     ((same :wat::time::Instant) (:wat::time::+ origin zero))
     ((delta :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds same)
        (:wat::time::epoch-seconds origin))))
    (:wat::test::assert-eq delta 0)))

(:wat::test::deftest :wat-tests::time::test-instant-sub-self-is-zero-duration
  ()
  (:wat::core::let*
    (((t :wat::time::Instant) (:wat::time::at 1000000))
     ((d :wat::time::Duration) (:wat::time::- t t)))
    ;; Same instant - itself = 0 ns Duration.
    (:wat::test::assert-eq (:wat::core::show d) "<Duration 0ns>")))


;; ─── Arc 097 slice 3 — `ago` / `from-now` composers ────────────────
;;
;; ActiveSupport-flavored. (ago d) = (- (now) d). (from-now d) =
;; (+ (now) d). Both take Duration; return Instant relative to now.

(:wat::test::deftest :wat-tests::time::test-ago-is-before-now
  ()
  (:wat::core::let*
    (((past :wat::time::Instant)
      (:wat::time::ago (:wat::time::Hour 1)))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ;; Past minus now should be a NEGATIVE duration normally, but
     ;; per §2 Durations are non-negative. So we reverse: now - past
     ;; should produce a positive Duration.
     ((elapsed :wat::time::Duration) (:wat::time::- now-i past)))
    ;; Elapsed should be at least 3,599,000,000,000 ns (just under
    ;; 1 hour, allowing for the few microseconds between the two
    ;; `now` calls). Asserting >= 3_599_000_000_000.
    (:wat::test::assert-eq
      (:wat::core::>=
        (:wat::core::-
          (:wat::time::epoch-nanos now-i)
          (:wat::time::epoch-nanos past))
        3599000000000)
      true)))

(:wat::test::deftest :wat-tests::time::test-from-now-is-after-now
  ()
  (:wat::core::let*
    (((future :wat::time::Instant)
      (:wat::time::from-now (:wat::time::Hour 1)))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ;; Future - now should yield positive Duration ~ 1 hour.
     ((elapsed-ns :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-nanos future)
        (:wat::time::epoch-nanos now-i))))
    ;; At least 3_599_000_000_000 ns (just under 1 hour, allowing
    ;; few microseconds for two `now` calls separating).
    (:wat::test::assert-eq (:wat::core::>= elapsed-ns 3599000000000) true)))

(:wat::test::deftest :wat-tests::time::test-ago-zero-equals-now
  ()
  ;; (ago (Hour 0)) = (now). Tolerance: same-second.
  (:wat::core::let*
    (((past :wat::time::Instant)
      (:wat::time::ago (:wat::time::Hour 0)))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((past-s :wat::core::i64) (:wat::time::epoch-seconds past))
     ((now-s :wat::core::i64) (:wat::time::epoch-seconds now-i)))
    ;; Same second (or off-by-one if the test crosses a second
    ;; boundary). Asserting absolute delta <= 1.
    (:wat::test::assert-eq
      (:wat::core::<= (:wat::core::- now-s past-s) 1)
      true)))

(:wat::test::deftest :wat-tests::time::test-from-now-zero-equals-now
  ()
  (:wat::core::let*
    (((future :wat::time::Instant)
      (:wat::time::from-now (:wat::time::Day 0)))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((delta-s :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds future)
        (:wat::time::epoch-seconds now-i))))
    (:wat::test::assert-eq (:wat::core::<= delta-s 1) true)))


;; ─── Arc 097 slice 4 — unit-ago / unit-from-now sugars ──────────────
;;
;; 14 sugars. (hours-ago 1) reads cleaner than
;; (:wat::time::ago (:wat::time::Hour 1)) at every callsite.
;; Equivalence at output: (hours-ago N) ≡ (ago (Hour N)).

(:wat::test::deftest :wat-tests::time::test-hours-ago-equivalent-to-ago-hour
  ()
  (:wat::core::let*
    (((via-sugar :wat::time::Instant) (:wat::time::hours-ago 1))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((delta-via-sugar :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-nanos now-i)
        (:wat::time::epoch-nanos via-sugar))))
    ;; (hours-ago 1) is now - 1h. Delta should be ~3.6e12 ns.
    (:wat::test::assert-eq
      (:wat::core::and
        (:wat::core::>= delta-via-sugar 3599000000000)
        (:wat::core::<= delta-via-sugar 3601000000000))
      true)))

(:wat::test::deftest :wat-tests::time::test-days-from-now-is-future
  ()
  (:wat::core::let*
    (((future :wat::time::Instant) (:wat::time::days-from-now 1))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((delta-s :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds future)
        (:wat::time::epoch-seconds now-i))))
    ;; ~86,400 seconds in a day, give or take a tick.
    (:wat::test::assert-eq
      (:wat::core::and
        (:wat::core::>= delta-s 86399)
        (:wat::core::<= delta-s 86401))
      true)))

(:wat::test::deftest :wat-tests::time::test-minutes-ago
  ()
  (:wat::core::let*
    (((past :wat::time::Instant) (:wat::time::minutes-ago 5))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((delta-s :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds now-i)
        (:wat::time::epoch-seconds past))))
    ;; 5 minutes = 300 seconds.
    (:wat::test::assert-eq
      (:wat::core::and
        (:wat::core::>= delta-s 299)
        (:wat::core::<= delta-s 301))
      true)))

(:wat::test::deftest :wat-tests::time::test-seconds-from-now
  ()
  (:wat::core::let*
    (((future :wat::time::Instant) (:wat::time::seconds-from-now 60))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((delta-s :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds future)
        (:wat::time::epoch-seconds now-i))))
    (:wat::test::assert-eq
      (:wat::core::and
        (:wat::core::>= delta-s 59)
        (:wat::core::<= delta-s 61))
      true)))

(:wat::test::deftest :wat-tests::time::test-zero-hours-ago-is-roughly-now
  ()
  (:wat::core::let*
    (((past :wat::time::Instant) (:wat::time::hours-ago 0))
     ((now-i :wat::time::Instant) (:wat::time::now))
     ((delta-s :wat::core::i64)
      (:wat::core::-
        (:wat::time::epoch-seconds now-i)
        (:wat::time::epoch-seconds past))))
    ;; Same second (or off-by-one if crossing a boundary).
    (:wat::test::assert-eq (:wat::core::<= delta-s 1) true)))
