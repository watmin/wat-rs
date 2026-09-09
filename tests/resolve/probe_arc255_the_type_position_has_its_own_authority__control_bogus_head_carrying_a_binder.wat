;; ★★★ GUARD — a BOGUS CALL HEAD that happens to carry a `:-` type binder.
;;
;; `(:wat::core::HashSet :- [T] "a" "b")` is a CONSTRUCTOR CALL whose head is a genuine call
;; head that merely carries an explicit type binder. So "the head of a `:-` form is type
;; syntax" is FALSE, and an implementation that exempts the head from call-head validation
;; lets an unresolvable head through in silence.
;;
;; ⛔ `walk.rs:87` ALREADY skips this shape (`is_type_reference`). `normalize` having NO such
;; guard is, today, the ONLY thing that refuses this program. Measured: clean main reports
;; `UnresolvedReferences` naming `:my::app::totally-bogus`, EXIT=1; the first strike at
;; stone 2 returned EXIT=0.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (my.app/totally-bogus :- [wat.type/i64] 1)))
