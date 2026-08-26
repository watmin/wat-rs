;; tests/types/probe_arc293_structtype_primitive.wat — co-located fixture
;;
;; Arc 293.2-parity — the :wat::core::structtype primitive.
;; RED at HEAD: :wat::core::structtype is an unknown declaration head.

(:wat::core::structtype :my::Point
  [x <- :wat::core::i64  y <- :wat::core::i64])

;; The driver, restored 2026-08-16 (it was deleted by 3cd00fbb, arc 170's :user::main
;; wall, leaving this fixture hollow and its test passing on nothing for 37 days).
;;
;; ⛔ IT MUST USE THE PRIME `:my::Point'`, NOT the bare kwargs name. That is this
;; probe's whole subject. `defstruct` expands to TWO forms (src/macros/parse.rs:266):
;;   (:wat::core::structtype ~@args)  +  (:wat::core::defmacro ~fqdn-bare-kw ...)
;; The PRIMITIVE registers the type and mints `T'` + the field accessors; the bare
;; kwargs name is a MACRO the outer `defstruct` emits. So `(:my::Point :x 3 :y 4)`
;; here would be testing `defstruct`'s surface while claiming to test the primitive —
;; and would fail with UnresolvedReference on a call head that structtype never
;; promised. The primitive's own construction surface is the prime.
(:wat::core::defn :probe::drive [] -> :wat::core::i64
  (:wat::i64::+
    (:my::Point/x (:my::Point' 3 4))
    (:my::Point/y (:my::Point' 3 4))))
