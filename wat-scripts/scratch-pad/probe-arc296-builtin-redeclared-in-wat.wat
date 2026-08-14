;; Arc 296 — THE TRAP-CHECK for "move the builtin records into the corpus".
;;
;; `:wat::kernel::Location` is registered TODAY by Rust (`register_builtin_types`,
;; src/types.rs:1465) as nature=Record with fields [file :String, line :i64, col :i64].
;; The stone wants that declaration to live in wat, with the Rust registration GENERATED
;; from this very form.
;;
;; The whole risk is one question: does re-declaring it in wat hit `TypeEnv::register`'s
;; `Existing::Equivalent` arm (a byte-equivalent re-declaration is a NO-OP, arc 054) or
;; its `Divergent` arm (`DuplicateType`, and the stdlib stops loading)?
;;
;; This form is transcribed to match the Rust literal EXACTLY. If it type-checks clean,
;; the parser and the hand-written builtin agree and the migration is mechanical.
;; If it raises DuplicateType, the two DISAGREE — and that disagreement is a defect that
;; exists TODAY, independent of this stone: Rust believes something about this type that
;; wat's own reader would not.

(:wat::core::defrecord :wat::kernel::Location
  [file <- :wat::core::String
   line <- :wat::core::i64
   col  <- :wat::core::i64])
