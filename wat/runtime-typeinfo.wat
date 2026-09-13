;; wat/runtime-typeinfo.wat — arc 296 L: the row `:wat::runtime::type-of` answers with.
;;
;; One declared row covering every TypeDef kind (Aggregate · Enum · Newtype · Alias ·
;; Union · Surface). Rust sources the registrations via wat_record_from! /
;; wat_enum_register_from! — this file is the source of truth, not a mirror.
;;
;; Loads AFTER Record.wat: TypeInfo / TypeField / TypeVariant are defrecords
;; (defrecord's expansion calls Record::def at eval time). The closed-domain
;; enums are defenum (a builtin) and live here so the whole row is one file.

;; Discriminant of TypeDef — the kind the row is about. `:Builtin` and
;; `:Marker` are membership without a TypeDef (`type-of` answers them so
;; it agrees with `is-type?`).
(:wat::core::defenum :wat::runtime::TypeKind :wat::enum::Pure
  :Aggregate
  :Enum
  :Newtype
  :Alias
  :Union
  :Surface
  :Builtin
  :Marker)

;; Aggregate / surface nature (the holder trit + Peer). Mirrors `Nature`.
(:wat::core::defenum :wat::runtime::TypeNature :wat::enum::Pure
  :Struct
  :Record
  :HolonRecord
  :Peer)

;; Enum purity (not the verb-axis `:wat::runtime::Purity`). Mirrors `types::Purity`.
(:wat::core::defenum :wat::runtime::TypePurity :wat::enum::Pure
  :Pure
  :Impure)

;; One declared field — name in declaration order, type as a WatAST form
;; (the same rendering `field-types-of` already uses).
(:wat::core::defrecord :wat::runtime::TypeField
  [name <- :wat::core::keyword
   type <- :wat::WatAST])

;; One enum variant — unit variants have an empty `fields` vector.
(:wat::core::defrecord :wat::runtime::TypeVariant
  [name <- :wat::core::keyword
   fields <- (:wat::core::Vector :- [:wat::runtime::TypeField])])

;; One surface member. A Method reports its parameter names (declaration
;; order) and return type; the full ArgSpec is not a wat value.
(:wat::core::defenum :wat::runtime::TypeSurfaceMember :wat::enum::Pure
  :Field  [name <- :wat::core::keyword  type <- :wat::WatAST]
  :Method [name <- :wat::core::keyword
           params <- (:wat::core::Vector :- [:wat::core::keyword])
           ret <- :wat::WatAST])

;; Kind-appropriate body. A Newtype with only `:inner` is an ANSWER (kind +
;; wrapped type), not an omission — "nothing else to say" lives in the row.
(:wat::core::defenum :wat::runtime::TypeBody :wat::enum::Pure
  :Aggregate [nature <- :wat::runtime::TypeNature
              fields <- (:wat::core::Vector :- [:wat::runtime::TypeField])]
  :Enum      [purity <- :wat::runtime::TypePurity
              variants <- (:wat::core::Vector :- [:wat::runtime::TypeVariant])]
  :Newtype   [inner <- :wat::WatAST]
  :Alias     [expr <- :wat::WatAST]
  :Union     [members <- (:wat::core::Vector :- [:wat::WatAST])]
  :Surface   [nature <- (:wat::core::Option :- [:wat::runtime::TypeNature])
              members <- (:wat::core::Vector :- [:wat::runtime::TypeSurfaceMember])]
  ;; A builtin's structure, parameters included, is not declared anywhere
  ;; (`builtin_names` holds names only) — so the row's `type-params` is empty
  ;; and the body has nothing else to say.
  :Builtin []
  :Marker  [children <- (:wat::core::Vector :- [:wat::core::keyword])])

;; THE row. `type-params` is on the row (every TypeDef carries them), not
;; buried in a kind-specific body. `body` is the rest.
(:wat::core::defrecord :wat::runtime::TypeInfo
  [name <- :wat::core::keyword
   kind <- :wat::runtime::TypeKind
   type-params <- (:wat::core::Vector :- [:wat::core::String])
   body <- :wat::runtime::TypeBody])

;; Outcome of `:wat::runtime::declared-types`. `Ok` carries the types the
;; program's declarations added to a FRESH copy of the stdlib registry.
;; `Refused` names the form that could not register and why — never a panic,
;; never a silent drop.
(:wat::core::defenum :wat::runtime::DeclaredTypes :wat::enum::Pure
  :Ok [types <- (:wat::core::Vector :- [:wat::runtime::TypeInfo])]
  :Refused [form <- :wat::WatAST
            cause <- :wat::core::String])
