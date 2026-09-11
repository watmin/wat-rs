;; The builder's question, pinned (arc 296 stone ③b-i / DESIGN-the-variant-name-is-
;; COMPOSED-never-TYPED.md): a `defn` and an enum variant's constructor path want the
;; SAME name. This order: the `defn` (spelled as the composed variant name) comes first,
;; ahead of the `defenum` on the next line.
;;
;; ⚠ arc 255 ③b-ii (the dot-flip) residue — NOT A REPAIR. Pre-flip the composed name was
;; `Foo::Bar`, an ordinary caller-typeable name, and this file proved `DuplicateDefine`.
;; Post-flip the composed name is `Foo.Bar` — a DOTTED caller-typed name — and H-1 refuses
;; it OUTRIGHT as `DottedName`, before the `defenum` below is ever reached. The collision
;; this file was built to demonstrate is now structurally unconstructible, which is the
;; rung ABOVE catching it; see the sibling `.rs`'s module doc.
(:wat::core::defn :my::app::Foo.Bar [] -> :wat::core::i64 1)
(:wat::core::defenum :my::app::Foo :wat::enum::Pure :Bar)
