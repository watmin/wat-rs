;; wat/list.wat — :wat::list::* — list operations.
;;
;; Forward-looking namespace per arc 109's wind-down direction
;; ("we need to move things to :wat::list::* then we can mirror
;; that stuff for lazy seqs"). Currently houses one alias —
;; :wat::list::reduce → :wat::core::foldl — using arc 143's
;; :wat::runtime::define-alias macro.
;;
;; Future :wat::core::foldl → :wat::list::foldl rename in a
;; follow-on arc; this alias's TARGET updates without touching
;; the alias's NAME.

(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)
