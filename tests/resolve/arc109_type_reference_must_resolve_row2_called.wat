;; tests/resolve/arc109_type_reference_must_resolve_row2_called.wat — row 2★★, THE STONE.
;;
;; Same phantom as row 1, but WITH a caller. Before this stone: the resolver never touches
;; type positions, so this reached `check_program`, which treated the phantom as a real,
;; distinct type and reported `TypeMismatch` naming PARAMETER #1 — blaming the caller for a
;; defect in the declaration. This is the case the resolve/check precedence
;; (`freeze.rs::resolve_error_names_a_phantom_type`) must not let a `TypeMismatch` outrank:
;; the declared-type finding must win and name the TYPE, not the call site.
(:wat::core::defn :user::f [x <- :user::NoSuchType] -> :wat::core::i64 0)
(:wat::core::defn :user::main [] -> :wat::core::i64 (:user::f 5))
