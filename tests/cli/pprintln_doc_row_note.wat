;; Row 6 — a non-Row record with a multi-line string stays escaped.
(wat.core/defrecord probe/Note
  [text :- wat.type/String])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/pprintln
    (probe/Note :text "line one\nline two")))
