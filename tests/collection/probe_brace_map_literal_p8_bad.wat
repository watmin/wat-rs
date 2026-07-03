;; tests/collection/probe_brace_map_literal_p8_bad.wat
;; Probe 8: old bare-symbol struct-pattern form {outcome grace-residue} in let-binding
;; position must now error (arc 257.2 — WatAST::StructPattern deleted).
(:wat::core::defstruct :test214::PaperResult
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:test214::PaperResult "kept" 3.14)
     {outcome grace-residue} p]
    outcome))
