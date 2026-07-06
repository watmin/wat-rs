;; vigilatum: 2026-06-04T05:50:09Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(record-def)
;;
;; :wat::core::Record::def — BASE record macro.
;;
;; Defines a BASE record: struct_form only, NO holon_form.
;; The unmarked name is the cheap common case.
;;
;; :wat::holon::Record::def — HOLONIC record macro.
;;
;; Defines a HOLONIC record: struct_form + holon_form (opt-in for holon-ops).
;;
;; Substrate primitives consumed:
;;   BASE:
;;   - :wat::core::Record::of             — 2-arg constructor → Value::wat__Record (struct only)
;;   - :wat::core::Record/field-at        — positional field accessor
;;   HOLONIC:
;;   - :wat::holon::Record::of      — 3-arg constructor → Value::wat__holon__Record
;;   - :wat::core::Record/field-at        — positional field accessor (same; variant-agnostic)
;;
;; Flavor hierarchy (Liskov):
;;   :wat::core::Record                   — base parent (all records are :wat::core::Record)
;;   :wat::holon::Record            — holonic parent (inherits from :wat::core::Record)
;;   A func wanting :wat::core::Record    accepts BOTH base and holonic instances.
;;   A func wanting :wat::holon::Record accepts ONLY holonic instances.
;;
;; Expansion shape for BASE:
;;   (:wat::core::Record::def :myapp::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
;;
;; →
;;   (:wat::core::do
;;     ;; 1. recordtype declaration (parent = :wat::core::Record)
;;     (:wat::core::recordtype :myapp::Pt :wat::core::Record [x y])
;;
;;     ;; 2. Constructor (2-arg :wat::core::Record::of — no holon_form)
;;     (:wat::core::defn :myapp::Pt [x <- :wat::core::i64  y <- :wat::core::i64] -> :wat::core::Record
;;       (:wat::core::Record::of
;;         :myapp::Pt
;;         [x y]))
;;
;;     ;; 3. Per-field accessor (one per field), receiver class-safety guarded:
;;     ;;    field-at runs only after (= (type v) :myapp::Pt) is checked.
;;     (:wat::core::defn :myapp::Pt/x [v <- :wat::core::Record] -> :wat::core::i64
;;       (:wat::core::Record/field-at <class-checked v> 0))
;;
;;     ;; 4. Predicate (auto-minted elsewhere; NOT emitted by this macro)
;;     )
;;
;; Expansion shape for HOLONIC:
;;   (:wat::holon::Record::def :myapp::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
;;
;; →
;;   (:wat::core::do
;;     ;; 1. recordtype declaration (parent = :wat::holon::Record)
;;     (:wat::core::recordtype :myapp::HPt :wat::holon::Record [x y])
;;
;;     ;; 2. Constructor (3-arg :wat::holon::Record::of — with holon_form)
;;     (:wat::core::defn :myapp::HPt [x <- :wat::core::i64  y <- :wat::core::i64] -> :wat::holon::Record
;;       (:wat::holon::Record::of
;;         :myapp::HPt
;;         [x y]
;;         (:wat::holon::Bind ...)))
;;
;;     ;; 3. Per-field accessors (one per field; class-safety guarded, predicate
;;     ;;    auto-minted elsewhere — same as BASE)
;;     )
;;
;; Naming rules (derived at macro-expand time via keyword/to-string + string manipulation):
;;
;;   | Input FQDN            | Constructor           | Predicate                  | Classifier string      |
;;   |-----------------------|-----------------------|----------------------------|------------------------|
;;   | :myapp::Pt            | :myapp::Pt            | :myapp::is-Pt?             | "myapp::Pt"            |
;;   | :awesome::lib::Sensor | :awesome::lib::Sensor | :awesome::lib::is-Sensor?  | "awesome::lib::Sensor" |
;;
;; (The Predicate column is shown for FQDN-derivation reference; the predicate is
;;  auto-minted elsewhere, not emitted by this macro.)
;;
;; Accessor naming: <class-fqdn>/<field-name> as keyword (e.g. :myapp::Pt/x).
;; Accessor signature: [v <- :wat::core::Record] -> :<declared-field-type>.
;;
;; FQDN doctrine (feedback_fqdn_is_the_namespace): users declare their own namespace.
;; The macro NEVER inserts into :user::* or any auto-namespace.
;;
;; Accessor bodies are class-safety guarded: each runs (:wat::core::Record/field-at …)
;; only after checking (= (type v) <fqdn>), panicking with a "got class …" message
;; on a mismatched receiver (see the accessor expansion in the macro body below).
;; Field-type constraints are NOT enforced at expand time. No aliases, no single-arg
;; form — users MUST provide the field vector.

;; ─── BASE macro (:wat::core::Record::def) ──────────────────────────────────────────

;; Arc 294.c.2a — base defrecord macro routes through aggregate-new (the ONE
;; nature-dispatched ctor). Drop the :wat::core::Record::of wrapper + field-extraction
;; let; bare field syms splice directly as positional args to aggregate-new.
;; Arc 291 hygiene preserved: raw-ch/nf/syms dance keeps scope-tagged AST nodes.
(:wat::core::defmacro :wat::core::defrecord
  [fqdn   <- :wat::WatAST
   fields <- :wat::WatAST]
  -> :wat::WatAST
  ;; Arc 293 surface-splice — the constructor `defn` is DELETED from this macro. The ctor
  ;; is now minted (for EVERY aggregate nature) in `register_aggregate_methods` (runtime.rs)
  ;; from the REGISTERED fields — so `~@:Surface` splices in the field vector are expanded
  ;; (at type registration) BEFORE the ctor is built, and records get splice for free.
  ;; The old expand-time `raw-ch/nf/syms` groups-of-3 walk (registry-blind, choked on `~@`)
  ;; is gone along with the whole lazy-seq / arc-118 concern.
  ;;
  ;; The macro now expands to the bare `recordtype` decl — NO `do` wrapper. A `do` wrapping
  ;; only a type decl would be emptied by `register_types` (which strips type decls from a
  ;; `do` body), leaving `(:wat::core::do)` → "do requires at least one form".
  `(:wat::core::recordtype ~fqdn :wat::core::Record
     [~@fields]))

;; Arc 293.R2.2 — accessor emission removed from BASE macro.
;; register_aggregate_methods (runtime.rs) now mints all field accessors for
;; every Aggregate nature (Struct + Record + HolonRecord): bare name, struct-field
;; body, type_params-aware.  The ctor defn above stays exactly as-is.

;; ─── HOLONIC macro (:wat::holon::Record::def) ────────────────────────────────

;; Arc 294.c.2a — holonic defrecord macro routes through aggregate-new (the ONE
;; nature-dispatched ctor). The entire hologram quasiquote (former lines ~157-197)
;; is DELETED — build_holon_hologram in Rust now derives the hologram internally.
;; Arc 291 hygiene preserved: raw-ch/nf/syms dance keeps scope-tagged AST nodes.
(:wat::core::defmacro :wat::holon::defrecord
  [fqdn   <- :wat::WatAST
   fields <- :wat::WatAST]
  -> :wat::WatAST
  ;; Arc 293 surface-splice — constructor `defn` DELETED (see the BASE macro above). The
  ;; holon ctor is minted in `register_aggregate_methods` from the registered fields; the
  ;; `aggregate-new` body is nature-blind and derives the hologram internally for HolonRecord.
  ;; Bare `recordtype` decl — NO `do` wrapper (see the BASE macro's note).
  `(:wat::core::recordtype ~fqdn :wat::holon::Record
     [~@fields]))

;; Arc 293.R2.2 — accessor emission removed from HOLONIC macro.
;; register_aggregate_methods (runtime.rs) now mints all field accessors for
;; every Aggregate nature (Struct + Record + HolonRecord): bare name, struct-field
;; body, type_params-aware.  The ctor defn above stays exactly as-is.
