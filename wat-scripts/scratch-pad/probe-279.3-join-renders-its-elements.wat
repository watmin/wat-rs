;; wat-scripts/scratch-pad/probe-279.3-join-renders-its-elements.wat
;;
;; THE WORKED EXEMPLAR for stone 279.3 (chain-D's join half). Every line here type-checks and runs
;; GREEN on the current substrate — this is the composition the strike moves into `wat/string.wat`,
;; not a sketch of one. Committed per FM 2-bis: a brief that names a non-trivial composition owes an
;; empirical probe, and the probe is what earns the right to assert it.
;;
;; PROVEN HERE, measured 2026-08-16:
;;   1. `T` BINDS INSIDE A LAMBDA nested in a parametric defn. This was the load-bearing unknown —
;;      `(:wat::core::fn [x <- T] ...)` inside `defn :user::join-ish<T>` resolves `T` correctly.
;;   2. `str` is TOTAL (stone 279.2, `25d9d015`) — an i64 element renders without a bound on `T`.
;;      That totality is the whole reason `T` needs no constraint; with a partial `str` this
;;      signature would require a type-variable bound, a form wat does not have.
;;   3. ★ A `String` ELEMENT RENDERS BARE — `"a-b"`, never `"\"a\"-\"b\""`. `mapv` applies `str` at
;;      TOP LEVEL per element, so 279.2's "nested strings stay quoted" rule does not fire here.
;;      This ANSWERS the contract question off the disk rather than by ruling: it is already Ruby's
;;      `ary.join(',') => "some,stringified,values"`.
;;   4. Delegating to the existing native over `Vector<String>` composes cleanly.
;;
;; ⚠ THE LAMBDA IS NOT STYLE — IT IS FORCED. `(mapv :wat::core::str xs)` does NOT type-check: a bare
;; intrinsic keyword is a `:wat::core::keyword`, not an `Fn(T)->U`, while a USER fn keyword IS.
;; See `docs/arc/2026/06/255-builtin-registry/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md`.
;; Do NOT "simplify" the lambda away — it will not compile, and the cause is arc 255's, not this
;; stone's.

(:wat::core::defn :user::join-ish<T>
  [sep <- :wat::core::String xs <- :wat::core::Vector<T>] -> :wat::core::String
  (:wat::core::string::join sep
    (:wat::core::mapv (:wat::core::fn [x <- T] -> :wat::core::String (:wat::core::str x)) xs)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; the row that does not work today through the public verb — numbers
    (:wat::kernel::println (:user::join-ish "," (:wat::core::Vector :wat::core::i64 1 2 3)))
    ;; the NON-VACUITY control: strings still work, and come back BARE (not re-quoted)
    (:wat::kernel::println (:user::join-ish "-" (:wat::core::Vector :wat::core::String "a" "b")))))
