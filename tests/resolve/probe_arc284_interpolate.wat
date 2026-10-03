(wat.core/defmacro user/mk [base :- wat.type/AST] :- wat.type/AST
  (wat.core/let
    [base-str (wat.core/ast-name base)
     full     (wat.string/interpolate "{b}::built" :b base-str)]
    (wat.core/first (wat.core/ast->children
      (wat.core/match (wat.core/read-string (wat.string/concat "\"" (wat.string/concat full "\""))) [wat.core/ReadOutcome.Forms {:forms __forms} __forms] [wat.core/ReadOutcome.Malformed {:cause __cause} (wat.core/macro-error (wat.string/concat "expand-time read-string failed: " (wat.core.Error/message __cause)))])))))
(wat.core/defn user/probe [] :- wat.type/String (user/mk hello))
(wat.core/defn user/runtime-interp [] :- wat.type/String
  (wat.string/interpolate "{a}::{b} {{lit}}" :a "x" :b 5))
