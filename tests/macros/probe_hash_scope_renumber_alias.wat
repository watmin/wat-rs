;; tests/macros/probe_hash_scope_renumber_alias.wat — co-located fixture for
;; probe_hash_scope_renumber.rs's macro_alias_expands_to_same_hash_as_direct_primitive.
;;
;; Program A: defines a macro alias and calls it. Textually distinct from the
;; companion probe_hash_scope_renumber_direct.wat — the defmacro + alias call
;; differs from the direct primitive call. After expansion, the defmacro form is
;; consumed by expand_all and the remaining output is one form:
;; (:my::prim 42 99 1 -1).
(wat.core/defmacro test/MyAlias
  [x :- wat.type/AST y :- wat.type/AST]
  :- wat.type/AST
  `(my/prim ~x ~y 1 -1))
(test/MyAlias 42 99)
