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

;; byte-at / byte-length -- O(1) indexing, and where they DIVERGE from the char-indexed pair
(:wat::core::defn :user::c13 [] -> :wat::core::i64 (:wat::string::byte-at "hello" 1))
(:wat::core::defn :user::c14 [] -> :wat::core::i64 (:wat::string::byte-length "hello"))
;; a two-byte character: byte-length is 3, length is 2, and byte 1 is a UTF-8 lead byte
(:wat::core::defn :user::c15 [] -> :wat::core::i64 (:wat::string::byte-length "aé"))
(:wat::core::defn :user::c16 [] -> :wat::core::i64 (:wat::string::length "aé"))
(:wat::core::defn :user::c17 [] -> :wat::core::i64 (:wat::string::byte-at "aé" 1))
;; out of range is loud, both directions
(:wat::core::defn :user::c18 [] -> :wat::core::i64 (:wat::string::byte-at "hello" 99))
(:wat::core::defn :user::c19 [] -> :wat::core::i64 (:wat::string::byte-at "hello" -1))
(:wat::core::defn :user::c20 [] -> :wat::core::i64 (:wat::string::byte-length ""))

;; byte-subs -- O(1) slicing, and what it refuses
(:wat::core::defn :user::c21 [] -> :wat::core::String (:wat::string::byte-subs "hello" 1 3))
(:wat::core::defn :user::c22 [] -> :wat::core::String (:wat::string::byte-subs "hello" 0 0))
;; "aé" is a 1-byte 'a' then a 2-byte 'é'; [0,1) and [1,3) are boundaries, [1,2) is not
(:wat::core::defn :user::c23 [] -> :wat::core::String (:wat::string::byte-subs "aé" 1 3))
(:wat::core::defn :user::c24 [] -> :wat::core::String (:wat::string::byte-subs "aé" 1 2))
(:wat::core::defn :user::c25 [] -> :wat::core::String (:wat::string::byte-subs "hello" 2 99))
