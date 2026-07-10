;; Negative fixture for probe_arc278_open_surface_dispatch.rs.
;;
;; The genuinely UNSOUND shape the return-type-soundness strike closes: a
;; `defclause` whose concrete-satisfier clauses are both NARROWING matches for
;; an open-surface arg (`:probe::Reason`) but declare DIFFERENT return types
;; (`String` vs `i64`). Before the fix, first-match-wins picked clause A's
;; return type (`String`) statically while the runtime could dispatch to
;; clause B (`i64`) — a program that compiles clean and hands a `string::concat`
;; call an `i64` at runtime.
;;
;; After the fix this must be rejected at CHECK time with
;; `CheckErrorKind::AmbiguousClauseReturnAtCallSite`, naming the incompatible
;; return types — never a runtime `TypeMismatch`. This file must NEVER
;; successfully start up; `probe_arc278_open_surface_dispatch.rs` asserts the
;; specific error variant + message.

(:wat::core::defsurface :probe::Reason :nature :wat::core::Record :features [])
(:wat::core::defrecord  :probe::A [x <- :wat::core::i64])
(:wat::core::defrecord  :probe::B [y <- :wat::core::i64])

;; DIFFERENT return types across the two concrete-satisfier clauses:
(:wat::core::defclause :probe::describe
  ([r <- :probe::A] -> :wat::core::String "a-string")
  ([r <- :probe::B] -> :wat::core::i64    42))

(:wat::core::defn :probe::as-reason [r <- :probe::B] -> :probe::Reason r)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [reason (:probe::as-reason (:probe::B 1))
     result (:probe::describe reason)]
    (:wat::kernel::println (:wat::core::string::concat "got: " result))))
