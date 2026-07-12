;; tests/types/probe_arc293_self_explicit.wat — co-located fixture (arc 293 K0c, the self-reference cycle-guard)
;;
;; A surface method with EXPLICIT `self <- :TheSurface` (the surface names itself — a standard recursive
;; type). At HEAD this STACK-OVERFLOWS: `struct_satisfies_surface` (surface.rs:83) compares the member's
;; self-type (= the surface) against a satisfier's self-type via `is_assignable`, which re-enters
;; satisfaction of the surface, which checks the self-type again — infinite recursion. (Bare `[self]`
;; left `fixed_params` empty, so the arg-type check was skipped and the cycle never fired.)
;;
;; GREEN after K0c: position 0 (self) is SKIPPED in the method arg-type comparison — self is the
;; receiver, tautologically the surface; it must never be re-checked.

(:wat::core::defsurface :se::Named :nature :wat::core::Record
  :features [(name [self <- :se::Named] -> :wat::core::String)])

(:wat::core::defrecord :se::Person [name <- :wat::core::String])   ; the `name` field accessor backs the method member

(:wat::core::defn :se::greet [x <- :se::Named] -> :wat::core::String (:se::Named/name x))

(:wat::core::defn :se::demo [] -> :wat::core::String (:se::greet (:se::Person :name "bob")))
