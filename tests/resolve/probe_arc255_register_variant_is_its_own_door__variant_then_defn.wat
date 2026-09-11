;; The builder's question, pinned — the OTHER order: the enum's variant ctor path
;; (`:my::app::Foo.Bar`, composed by `register_variant` from `(:my::app::Foo, Bar)`)
;; registers first, so the `defn` typed literally under that same name comes second.
;;
;; ⚠ arc 255 ③b-ii (the dot-flip) residue — NOT A REPAIR. Pre-flip the composed name was
;; `Foo::Bar` and this file proved `DuplicateDefine` in this order too, symmetrically with
;; the sibling `defn_then_variant.wat`. Post-flip the composed name is `Foo.Bar` — the
;; `defn` below types a DOTTED name, and H-1 refuses it OUTRIGHT as `DottedName`,
;; regardless of the enum already having registered its variant. Both orders now raise the
;; identical `DottedName`, not `DuplicateDefine`; see the sibling `.rs`'s module doc.
(:wat::core::defenum :my::app::Foo :wat::enum::Pure :Bar)
(:wat::core::defn :my::app::Foo.Bar [] -> :wat::core::i64 1)
