(wat.core/do
  (wat.core/defstruct my/State
    [counter :- wat.type/i64])
  (wat.core/defn my/main [] :- my/State (my/State :counter 42)))
