;; probe-failure-record-ctor.wat — Phase 0 of arc 278 Strike A (BRIEF-failure-canonical-constructor.md).
;;
;; Ground the bare-name record ctor for the builtin `:wat::kernel::Failure` type. Failure is
;; canonically a Record (Nature::Record — arc 293.W.2b), but several sites hand-roll it via
;; `:wat::core::struct-new` (wrong nature: Struct), which `Failure/message` (a Record accessor)
;; can't read.
;;
;; DEVIATION FROM THE BRIEF'S LITERAL SNIPPET: the brief's positional form
;; `(:wat::kernel::Failure "hello" :wat::core::None ...)` does NOT type-check — arc-294 9a's
;; kwargs flip retired bare-positional aggregate construction wholesale (builtin records
;; included, not just user `defrecord`s): "bare-positional construction of :wat::kernel::Failure
;; is retired (the bare name is the kwargs macro)". This is doctrinally correct and expected
;; (memory: "bare aggregate name = kwargs macro, prime `:T'` = generated-code-only"; kwargs are
;; categorically superior to positional for hand-written source). The PROVEN form is the KWARGS
;; ctor below — same bare head `:wat::kernel::Failure`, not struct-new, not a Value-erasure, not
;; the generated-code-only `Failure'` prime. Field names grounded exact from the builtin's
;; registration (src/types.rs): error/frames/actual/expected (arc 278 the string-wrap
;; annihilation — the stored message/location fields were REMOVED; Failure carries the raised
;; `:wat::core::Error` STRUCTURALLY in a mandatory `error` field, and `Failure/message` is now a
;; DERIVED accessor reading `error.message`). Order/types cross-checked against `message_only_failure`.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [f (:wat::kernel::Failure
         :error (:wat::core::Fault/of "hello")
         :frames (:wat::core::Vector :- [:wat::kernel::Frame])
         :actual :wat::core::None
         :expected :wat::core::None)]
    (:wat::kernel::println (:wat::kernel::Failure/message f))))
