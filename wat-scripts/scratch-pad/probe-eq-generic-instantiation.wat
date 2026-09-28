;; probe-eq-generic-instantiation.wat — arc 255 Stone 1c-b-iii, the load-bearing
;; generic-instantiation experiment.
;;
;; QUESTION: `is_type_equatable` admits a bare, unresolved rigid type param (`:T`, via
;; `is_type_param_letter`) so a GENERIC body comparing two `:T` values type-checks once,
;; for any choice of T (this is what keeps `wat/test.wat`'s `assert-eq :- [T] [actual <-
;; :T expected <- :T]` compiling). But every CONCRETE call site is checked separately,
;; against the callee's DECLARED parameter types (`:T`), not by re-running
;; `infer_equality`'s domain gate under the instantiation. So: does instantiating a
;; generic `=`-body at a type `values_equal` has no arm for (`:wat::core::fn`) get
;; refused at the call site, or does it slip through `--check` to a runtime raise?
;;
;; `eq-generic`'s OWN body — `(:wat::core::= a b)` with `a`, `b` : `:T` — is checked
;; exactly ONCE, generically, with `:T` admitted by the deferred-to-runtime rule. Calling
;; it with two `:wat::core::fn` arguments only has to satisfy ordinary call-site argument
;; unification (`a <- :T`, `b <- :T` — both args unify with the SAME fresh `T`, which two
;; structurally-identical Fn types do). `infer_equality`/`is_type_equatable` are never
;; invoked again for this call — they ran once, at `eq-generic`'s declaration, against
;; the unresolved `:T`.
;;
;; MEASURED RESULT (2026-09-03): `--check` exits 0 — the checker ADMITS this program.
;; Running it raises `#wat.runtime/TypeMismatch {:op ":wat::core::=" :expected "matching
;; comparable pair" :got ...wat::core::fn...}` — `values_equal`'s `_ => None` arm, exactly
;; the same raise `probe-core-eq-is-partial.wat` measures at the DIRECT call site, except
;; here it happens INSIDE a generic body the gate could not see through.
;;
;; CONCLUSION: `:wat::core::=` is `Total` at concrete call sites (this gate closes that
;; hole) but the hole SURVIVES inside a generic body instantiated at an unequatable type.
;; `=`/`not=` grade `@Totality Partial` — same verdict as before this stone, for a
;; narrower and now-precisely-located reason (the type-var door, not an ungated `Fn` domain).
(:wat::core::defn :user::eq-generic :- [[T :< :wat::core::Equatable]] [a <- :T b <- :T] -> wat.type/bool
  (:wat::core::= a b))

;; The call on two functions is tests/types/probe_arc255_56_eq_generic_fn.wat.bad.
;; The bound refuses it: T is Equatable, a function is not.
