;; wat/list.wat — :wat::list::* — list operations.
;;
;; Forward-looking namespace per arc 109's wind-down direction
;; ("we need to move things to :wat::list::* then we can mirror
;; that stuff for lazy seqs"). Houses two opinionated aliases for
;; users who reach for `reduce` or `fold` — both delegate to the
;; atomic `:wat::core::foldl` primitive. (`foldl` and `foldr`
;; remain the atomic forms; `reduce` and `fold` are the helper
;; names users reach for from Clojure / Haskell / Lisp / JS / Python /
;; Ruby etc.)
;;
;; Future :wat::core::foldl → :wat::list::foldl rename in a
;; follow-on arc; the aliases' TARGET updates without touching
;; their NAMES.

(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)
(:wat::runtime::define-alias :wat::list::fold   :wat::core::foldl)
