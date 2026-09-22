;; Stone 255.12 — ONE type, TWO spellings: the re-declaration the registration
;; gate must call a NO-OP.
;;
;; `wat.type/X` and `wat.core/X` are one type (255.8 wired `type_denotation` into
;; eight places). `register_validated` was not one of them: it asked `e == &def`,
;; a RAW comparison of the stored `TypeDef`. A `wat.type/`-spelled field stores
;; `:wat::type::i64`, so every declaration re-delivered in the other spelling —
;; the arc-054 shape: a form baked into the binary by `include_str!` AND read off
;; disk — raised `DuplicateType`. That is what broke `wat/source.wat` under
;; conversion.
;;
;; Every pair below is the SAME declaration written twice, once per spelling.
;; Freezing this file must succeed. The negative halves live in the
;; `probe_arc255_12_redeclare_*.wat.bad` siblings — a gate that only ever
;; ACCEPTS is indistinguishable from a gate that has stopped firing.

;; ─── aggregate: a scalar field ───────────────────────────────────────────
(:wat::core::defrecord :p255_12::Rec [n <- :wat::core::i64])
(:wat::core::defrecord :p255_12::Rec [n <- :wat::type::i64])

;; ─── aggregate: a PARAMETRIC field — head AND argument both re-spelled ───
(:wat::core::defrecord :p255_12::Holder
  [xs <- (:wat::core::Vector :- [:wat::core::i64])])
(:wat::core::defrecord :p255_12::Holder
  [xs <- (:wat::type::Vector :- [:wat::type::i64])])

;; ─── enum: a tagged variant's field ─────────────────────────────────────
(:wat::core::defenum :p255_12::Enm :wat::enum::Pure
  :Ok [v <- :wat::core::i64]
  :Err)
(:wat::core::defenum :p255_12::Enm :wat::enum::Pure
  :Ok [v <- :wat::type::i64]
  :Err)

;; ─── typealias: the alias target ────────────────────────────────────────
(:wat::core::typealias :p255_12::Al :wat::core::i64)
(:wat::core::typealias :p255_12::Al :wat::type::i64)
