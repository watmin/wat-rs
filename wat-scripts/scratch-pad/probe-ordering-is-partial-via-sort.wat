;; probe-ordering-is-partial-via-sort.wat — arc 255, the counterexample that re-grades the
;; four orderings.
;;
;; ⛔ CLAIM REFUTED: Stone 1c-b-ii graded `<`/`>`/`<=`/`>=` `@Totality Total` on the reading that
;; `infer_ordering` gates every call through `is_type_orderable`, which excludes `Fn`. That gate
;; is real — a DIRECT `(:wat::core::< <fn> <fn>)` is refused at check time. But it has a door:
;;
;;     is_type_orderable:  TypeExpr::Var(_) => true,   // unresolved — defer to runtime
;;
;; A RIGID type param (`:- [T]`, spelled `:T`) is `TypeExpr::Path(":T")` and IS refused — which is
;; how the door was mistaken for dormant. An AUTO-GENERALISED one (arc 256 / Stone 251.7), such as
;; the `T` in `(:wat::core::Vector :- [T])`, is a real `Var` and sails through.
;;
;; `wat/core.wat`'s `sort` is exactly that shape: its 1-ary clause hands `sort$native` the
;; comparator `(:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool (:wat::core::< a b))`.
;; So sorting a Vector of functions TYPE-CHECKS, and `<` then meets a pair `values_compare` has
;; no arm for.
;;
;; THIS FILE TYPE-CHECKS. That is the whole assertion — the same shape
;; `probe-eq-generic-instantiation.wat` carries for `=`. It is not run: `sort` on this vector
;; raises, which is the point.

(:wat::core::typealias :user::IntFn [:wat::core::i64 :-> :wat::core::i64])

(:wat::core::defn :user::sort-fns
  [v <- (:wat::core::Vector :- [:user::IntFn])] -> (:wat::core::Vector :- [:user::IntFn])
  (:wat::core::sort v))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "probe-ordering-is-partial-via-sort: loaded"))
