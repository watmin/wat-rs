;; tests/types/probe_arc293_decl_a_aggregatetype.wat — co-located fixture (arc 293 decl-a)
;;
;; The declaration unification: ONE type-reg primitive `aggregatetype`, the holder DERIVED
;; from the parent's root (293 audit — holder is a passing policy; declaration is holder-agnostic).
;;   root_holder_of(:Parent):  :wat::core::Struct → Struct · :wat::core::Record → Record · :wat::holon::Record → HolonRecord
;; decl-a mints `aggregatetype` + `parse_aggregate` + the `:wat::core::Struct` lattice node
;; (structs repoint their root Value → Struct, behaviour-preserving since Struct <: Value).
;;
;; RED at HEAD: `:wat::core::aggregatetype` is unknown AND `:wat::core::Struct` is not a node.
;; GREEN after decl-a: a struct declared via the unified primitive registers (holder=Struct from
;; the :wat::core::Struct parent root), and its codegen'd ctor + accessor work.

;; A struct, declared via the ONE unified primitive — parent IS its holder root.
(:wat::core::aggregatetype :test::da::ST :wat::core::Struct
  [a <- :wat::core::i64  b <- :wat::core::i64])

;; The struct's bare ctor + field accessor are codegen'd over the registered Aggregate
;; (register_*_methods runs for every TypeDef::Aggregate). Construct + read field a.
(:wat::core::defn :user::da-st-a [] -> :wat::core::i64
  (:test::da::ST/a (:test::da::ST 7 8)))

;; A record, same unified primitive, parent = the Record holder root → holder Record.
(:wat::core::aggregatetype :test::da::BR :wat::core::Record
  [a <- :wat::core::i64  b <- :wat::core::i64])
