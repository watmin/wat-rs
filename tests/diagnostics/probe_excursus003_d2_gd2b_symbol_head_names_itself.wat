;; Excursus 003 strike D2, GD2b — a symbol-headed call names ITS OWN activation.
;;
;; `AUDIT-the-shape-of-an-error.md` § "Strike D landed", finding 2: `(f 1)`, with `f` a
;; local bound to a non-callable, used to raise `NotCallable` with its Rust frame named
;; `:wat::core::let` — the ENCLOSING special form that happened to be running when `f`
;; was bound, not the application that actually raised. `f`'s own binding has nothing to
;; do with the raise; the honest name is the head AS WRITTEN (`f`), per
;; `BRIEF-shape-strike-D2-every-raise-names-its-activation.md` item 1.

(:wat::core::defn :my::test::probe-notcallable [] -> :wat::core::i64
  (:wat::core::let [f 5]
    (f 1)))
