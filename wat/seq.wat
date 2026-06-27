;; vigilatum: 2026-06-04T02:28:55Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(list-fold-aliases)
;;
;; wat/seq.wat — :wat::seq::* — eager sequence operations.
;;
;; Two ergonomic aliases over the atomic fold primitive
;; `:wat::core::foldl`: `reduce` and `fold` are the names users reach
;; for from Clojure / Haskell / Lisp / JS / Python / Ruby. The alias
;; layer also insulates callers from a future move of the target — the
;; binding can be repointed without changing these names.

;; Stone 241.12 — migrated from :wat::runtime::define-alias to :wat::core::defalias.
(:wat::core::defalias :wat::seq::reduce :wat::core::foldl)
(:wat::core::defalias :wat::seq::fold   :wat::core::foldl)
