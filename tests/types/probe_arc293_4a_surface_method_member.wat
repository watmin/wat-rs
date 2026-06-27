;; tests/types/probe_arc293_4a_surface_method_member.wat — co-located fixture for the sibling probe (.rs).
;;
;; A `defsurface` with a METHOD member; a record satisfies it via a `:T/method` defn.
;; RED until 293.4a: `parse_defsurface` is field-only, so `(size [self] -> :i64)` is a malformed
;; member → the decl errors at load. GREEN at 293.4a: the method member parses, and `:geo::Box`
;; (exposing `:geo::Box/size`) structurally satisfies `:geo::Sized`, so `(needs-sized (Box 5))` checks.

;; surface with a single METHOD member (no fields)
(:wat::core::defsurface :geo::Sized
  (size [self] -> :wat::core::i64))

;; a record that backs `size` with a method (a defn :geo::Box/size)
(:wat::core::defrecord :geo::Box [w <- :wat::core::i64])
(:wat::core::defn :geo::Box/size [self <- :geo::Box] -> :wat::core::i64
  (:geo::Box/w self))

;; a consumer requiring :geo::Sized — a :geo::Box must be ACCEPTED here (it exposes size)
(:wat::core::defn :geo::needs-sized [s <- :geo::Sized] -> :wat::core::nil nil)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:geo::needs-sized (:geo::Box 5)))
