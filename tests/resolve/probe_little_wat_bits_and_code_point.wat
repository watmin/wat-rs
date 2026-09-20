;; Fixture for probe_little_wat_bits_and_code_point.rs -- the eight verbs the
;; `the-little-wat` branch adds, each driven at the case that would catch a
;; plausible WRONG implementation rather than at a case any of them would pass.

;; the three that are the same shape
(:wat::core::defn :user::c01 [] -> :wat::core::i64 (:wat::i64::bit-and 12 10))
(:wat::core::defn :user::c02 [] -> :wat::core::i64 (:wat::i64::bit-or 12 10))
(:wat::core::defn :user::c03 [] -> :wat::core::i64 (:wat::i64::bit-xor 12 10))

;; the one unary member: every bit of zero is one
(:wat::core::defn :user::c04 [] -> :wat::core::i64 (:wat::i64::bit-not 0))

(:wat::core::defn :user::c05 [] -> :wat::core::i64 (:wat::i64::bit-shift-left 1 10))

;; **sign-extending**: -7 >> 1 is -4, not 9223372036854775804. A logical shift
;; here would answer the latter, so this case tells the two apart.
(:wat::core::defn :user::c06 [] -> :wat::core::i64 (:wat::i64::bit-shift-right -7 1))

;; **zero-filling**: -1 is sixty-four ones, so >>> 60 leaves exactly four of
;; them. An arithmetic shift would answer -1. This is the whole reason the
;; unsigned verb exists as a separate name.
(:wat::core::defn :user::c07 [] -> :wat::core::i64
  (:wat::i64::unsigned-bit-shift-right -1 60))

;; **the count is masked to six bits**, so shifting by 64 is shifting by 0 --
;; which is what x86 and the JVM both do, and what clj inherits. An
;; implementation that saturated to zero would answer 0 here.
(:wat::core::defn :user::c08 [] -> :wat::core::i64 (:wat::i64::bit-shift-left 1 64))

;; code-point-at: indexed by CHARACTER, answering a number
(:wat::core::defn :user::c09 [] -> :wat::core::i64
  (:wat::string::code-point-at "hello" 1))
(:wat::core::defn :user::c10 [] -> :wat::core::i64
  (:wat::string::code-point-at "A" 0))

;; out of range is LOUD, not a silent zero
(:wat::core::defn :user::c11 [] -> :wat::core::i64
  (:wat::string::code-point-at "hello" 99))
(:wat::core::defn :user::c12 [] -> :wat::core::i64
  (:wat::string::code-point-at "hello" -1))
