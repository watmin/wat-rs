;; Fixture probe 14: fractal typeunion resolves transitively — :Baz = {:Foo, bool} where :Foo = {i64, f64}.
(wat.core/typeunion my/Foo [wat.type/i64 wat.type/f64])
(wat.core/typeunion my/Baz [my/Foo wat.type/bool])
(wat.core/defn my/identity [x :- my/Baz] :- my/Baz x)
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/do
    (my/identity 42)
    (my/identity 3.14)
    (my/identity true)
    nil))
