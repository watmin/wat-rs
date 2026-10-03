(wat.core/defn fix/unreadable-compound
  [p :- wat.type/String]
  :- wat.grep/Unreadable
  (wat.grep/Unreadable :file p :reason (wat.string/concat p "!") :line 0 :col 0))
