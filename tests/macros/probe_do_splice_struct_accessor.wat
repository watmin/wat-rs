(:wat::core::do
  (:wat::core::defstruct :my::State
    [counter <- :wat::core::i64])
  (:wat::core::defn :my::main [] -> :my::State (:my::State/new 42)))
