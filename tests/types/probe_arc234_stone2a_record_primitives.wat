;; tests/types/probe_arc234_stone2a_record_primitives.wat
;; Co-located fixture for probe_arc234_stone2a_record_primitives.rs (arc 234 Stone 234.2a).
;; Each :user::probe-N function corresponds to its Rust probe contract.
;;
;; Arc 296 G-1b — "finish the kill": `:wat::holon::Record::of` (the primitive this fixture used
;; to hand-build holograms through) was deleted, superseded by `aggregate-new` via the generated
;; defrecord constructor. Re-expressed per the brief: declare the two record types with
;; `:wat::holon::defrecord` and construct through their generated ctors. Every probe's SUBJECT —
;; construction shape, `:wat::core::type`, `Record/field-at`, hologram equality — is unchanged;
;; only how the input is built changed.

(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::holon::defrecord :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])

;; ─── Probe 1: construction returns :wat::holon::Record (HolonRecord aggregate) ──────
(:wat::core::defn :user::probe-1 [] -> :wat::holon::Record
  (:myapp::Voltage :magnitude 5.0))

;; ─── Probe 2: :wat::core::type returns class_fqdn ────────────────────────────
(:wat::core::defn :user::probe-2 [] -> :wat::core::String
  (:wat::core::let
    [v (:myapp::Voltage :magnitude 5.0)]
    (:wat::core::type v)))

;; ─── Probe 3: single-field struct_form ───────────────────────────────────────
(:wat::core::defn :user::probe-3 [] -> :wat::holon::Record
  (:myapp::Voltage :magnitude 42.0))

;; ─── Probe 4: multi-field construction ───────────────────────────────────────
(:wat::core::defn :user::probe-4 [] -> :wat::holon::Record
  (:myapp::Point :x 3 :y 4))

;; ─── Probe 5: Record/field-at positional access ──────────────────────────────
(:wat::core::defn :user::probe-5 [] -> :wat::core::i64
  (:wat::core::let
    [v (:myapp::Point :x 3 :y 4)]
    (:wat::core::Record/field-at v 1)))

;; ─── Probe 7: equality via holon_form ────────────────────────────────────────
(:wat::core::defn :user::probe-7 [] -> :wat::holon::Record
  (:myapp::Voltage :magnitude 5.0))
