;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Ngram)
;;
;; :wat::holon::Ngram — n-wise adjacency encoding.
;;
;; (Ngram n xs) = (Bundle (map (window xs n) Sequential))
;;
;; Slides a size-n window across xs, encodes each window with
;; Sequential (bind-chain), bundles every window's compound into one
;; composite holon.
;;
;; The macro expansion references :wat::holon::Sequential at parse
;; time — the macro-expander recursively expands it inside the
;; fn, so the emitted AST carries Sequential's let-foldl
;; directly with no runtime call hop.
;;
;; Edge cases: n <= 0 produces an empty bundle (zero vector);
;; n > xs.len() produces an empty bundle (no window fits).

;; Returns the Bundle's raw Result — caller handles capacity. Every
;; stdlib form that expands to Bundle inherits Bundle's Result wrap.
;; Callers either match explicitly or propagate with
;; `:wat::core::Result/try`.

;; Arc 118.2a — `map` flipped LAZY (returns Stream); `Bundle` needs a concrete
;; `(Vector :- [HolonAST])` eagerly, so the EXPANDED code uses `mapv` here. (This is a template
;; spliced into ordinary caller code, evaluated at normal runtime — not a macro-expansion-
;; time bootstrap site, so `mapv` is safe to reference directly.)
(:wat::core::defmacro :wat::holon::Ngram
  [n  <- :wat::WatAST
   xs <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::holon::Bundle
     (:wat::core::mapv
       (:wat::core::fn [window <- :wat::holon::Holons] -> :wat::holon::HolonAST
         (:wat::holon::Sequential window))
       (:wat::std::list::window ~xs ~n))))
