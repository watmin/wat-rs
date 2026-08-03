;; tests/types/probe_arc293_decl_b1_ctor_codegen.wat — co-located fixture (arc 293 decl-b.1)
;;
;; The ctor-codegen unification: the bare `:T` constructor is codegen'd for EVERY nature
;; (struct already is, via register_struct_methods; decl-b.1 extends it to record + holon).
;; Once the ctor comes from codegen for all, the defrecord/holon::defrecord macros stop
;; emitting a ctor `defn` — and the duplicated `syms`-extraction dance dies with it.
;;
;; The proof: a record declared via the RAW `recordtype` primitive (NO defrecord macro) gets
;; its bare ctor from CODEGEN. At HEAD a raw-recordtype record has accessors (R2.2) but NO ctor
;; (the ctor was macro-only) → `(:test::db::BR 7 8)` is unresolved. GREEN after decl-b.1.

;; A record via the RAW primitive — no macro, so any ctor MUST come from codegen.
(:wat::core::recordtype :test::db::BR :wat::core::Record [a <- :wat::core::i64  b <- :wat::core::i64])

;; Construct via the bare ctor (codegen'd) + read field a = 7.
(:wat::core::defn :user::db-br-a [] -> :wat::core::i64
  (:test::db::BR/a (:test::db::BR' 7 8)))

;; Same for a holon record via the raw primitive.
(:wat::core::recordtype :test::db::HR :wat::holon::Record [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::core::defn :user::db-hr-a [] -> :wat::core::i64
  (:test::db::HR/a (:test::db::HR' 7 8)))

;; The holon record built via the RAW primitive must be a REAL holon record — it must
;; carry a hologram (cosine with itself = 1.0). At HEAD the register_aggregate_methods
;; fallback builds it via :wat::core::Record::of (BASE ctor) → no hologram → this misbehaves.
;; decl-b.1 routes the fallback through aggregate-new (nature-dispatched) → hologram derived.
;; Arc 278 the cosine outcome wall — cosine now returns :wat::holon::CosineOutcome,
;; not a bare f64; the .rs side extracts the Similarity variant's field.
(:wat::core::defn :user::db-hr-cos [] -> :wat::holon::CosineOutcome
  (:wat::core::let [h (:test::db::HR' 7 8)]
    (:wat::holon::cosine h h)))
