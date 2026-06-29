(:wat::core::let []
  (:wat::core::defstruct :my::State
    [counter <- :wat::core::i64])
  (:wat::core::defn :my::main [] -> :my::State (:my::State 42)))
