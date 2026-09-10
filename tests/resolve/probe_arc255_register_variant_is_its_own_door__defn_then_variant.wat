;; The builder's question, pinned (arc 296 stone ③b-i / DESIGN-the-variant-name-is-
;; COMPOSED-never-TYPED.md): a `defn` and an enum variant's constructor path want the
;; SAME name. First-definer-wins never silently applies — the SECOND declaration must
;; raise `DuplicateDefine`, symmetrically in either definition order. This order: the
;; `defn` registers first, so the enum's variant ctor collides against it.
(:wat::core::defn :my::app::Foo.Bar [] -> :wat::core::i64 1)
(:wat::core::defenum :my::app::Foo :wat::enum::Pure :Bar)
