;; SPECIMEN — the-little-wat F-006. `wat.type/WatAST` resolves to :wat::core::WatAST, which does
;; not exist (the real type is :wat::WatAST). The checker refuses it, and the refusal's :location
;; names WAT-RS'S OWN src/check.rs instead of this file.
;; ⛔ Do not "fix" this program — the program being wrong is the point; the defect is WHERE the
;; diagnostic says the wrongness is.
(wat.core/defn u/kind [x :- wat.type/WatAST] :- wat.type/String
  (wat.core/ast-kind x))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (u/kind (wat.core/quote pear))))
