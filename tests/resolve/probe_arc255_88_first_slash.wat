;; Stone 255.88. The first `/` partitions namespace from name. A binder is
;; `{$bound, <whole spelling>}`. A body reference resolves locally first.
(wat.core/defn u/a/b [] :- wat.type/i64
  7)

(wat.core/defn u/pathological/name//foo [] :- wat.type/nil
  (wat.core/let
    [completely/insane 42]
    (wat.kernel/println completely/insane)))

(wat.core/defmacro probe.slash/introduced [] :- wat.type/AST
  (wat.core/quasiquote
    (wat.core/let
      [completely/insane 1]
      completely/insane)))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [foo/bar 0]
    (wat.kernel/println foo/bar))
  (wat.kernel/println (u/a/b))
  (u/pathological/name//foo)
  (wat.core/let
    [completely/insane 99]
    (wat.kernel/println (probe.slash/introduced))
    (wat.kernel/println completely/insane)))
