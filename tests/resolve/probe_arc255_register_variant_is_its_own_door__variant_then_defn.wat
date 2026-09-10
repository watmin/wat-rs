;; The builder's question, pinned — the OTHER order: the enum's variant ctor path
;; (`:my::app::Foo::Bar`, composed by `register_variant` from `(:my::app::Foo, Bar)`)
;; registers first, so the `defn` typed literally under that same name collides against
;; it. Both orders must raise `DuplicateDefine` — the composed door does not get a
;; first-mover advantage over a caller-typed name, or vice versa.
(:wat::core::defenum :my::app::Foo :wat::enum::Pure :Bar)
(:wat::core::defn :my::app::Foo.Bar [] -> :wat::core::i64 1)
