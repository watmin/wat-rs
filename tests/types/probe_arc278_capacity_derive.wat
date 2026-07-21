;; Arc 278 capacity stone 1 — `:wat::telemetry::framing-floor-of` ADAPTIVITY (disconfirming probe).
;;
;; RED at HEAD: `:wat::telemetry::framing-floor-of` is unknown ("unknown callee") — the whole file
;; fails to freeze (a check-time error, same class as the arc278 telemetry-records probe beside
;; this one). GREEN once wat/telemetry.wat ships the derive.
;;
;; The two `assert-true` calls below are TOP-LEVEL `def` bindings, not statements inside
;; `:user::main` — freeze evaluates every top-level `def`'s RHS eagerly (the same mechanism
;; `:wat::telemetry::LOG-MSG-CAPACITY` itself uses), but `:user::main` is never invoked by the
;; freeze pipeline this probe's `.rs` companion drives (`startup_beside` only freezes+checks — "the
;; wat binary['s]" job, not the test harness's). So the assertions MUST fire at load time to be
;; exercised at all. A failing `assert-true` panics (`assertion-failed!`) during its def's own
;; evaluation, surfacing as a freeze error exactly like the RED "unknown callee" does — so
;; `world.is_ok()` on the Rust side is the single, honest gate for both failure modes.
;;
;; Proves:
;;   1. ADAPTIVITY (the whole point) — RecB (RecA + one extra fixed `i64` field `:c`) yields a
;;      STRICTLY LARGER floor than RecA: the mechanism re-derives the floor from the LIVE field
;;      set (i64 value cost 20 + the `:c` key cost), no hand edits needed for tomorrow's schema.
;;   2. The real `:wat::telemetry::Log` floor is `>= 56` (the design doc's fixed-value-only sum:
;;      Uuid 36 + i64 20; the shipped derive additionally adds field-name-key + tag costs on top).
(:wat::core::defrecord :probe::RecA
  [a <- :wat::core::i64  b <- :wat::core::String])
(:wat::core::defrecord :probe::RecB
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::i64])

(:wat::core::def :probe::floor-a (:wat::telemetry::framing-floor-of :probe::RecA))
(:wat::core::def :probe::floor-b (:wat::telemetry::framing-floor-of :probe::RecB))
(:wat::core::def :probe::floor-log (:wat::telemetry::framing-floor-of :wat::telemetry::Log))

(:wat::core::def :probe::assert-adaptive
  (:wat::test::assert-true (:wat::core::> :probe::floor-b :probe::floor-a)))
(:wat::core::def :probe::assert-log-floor
  (:wat::test::assert-true (:wat::core::>= :probe::floor-log 56)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "arc278 capacity derive: adaptivity proven"))
