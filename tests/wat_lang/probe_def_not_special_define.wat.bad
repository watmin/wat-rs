;; Negative fixture: probe 4 — :wat::core::define is HARD CUT at startup-check.
;; Stone 241.11/241.16 — startup fails; does not reach runtime.
(:wat::core::defn :my::bad-define [] -> :wat::core::nil
  (:wat::core::define (:my::inner -> :wat::core::nil) :wat::core::nil))
