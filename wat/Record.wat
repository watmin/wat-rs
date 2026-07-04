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
;; holder-dispatched ctor). Drop the :wat::core::Record::of wrapper + field-extraction
;; let; bare field syms splice directly as positional args to aggregate-new.
;; Arc 291 hygiene preserved: raw-ch/nf/syms dance keeps scope-tagged AST nodes.
(:wat::core::defmacro :wat::core::defrecord
  [fqdn   <- :wat::WatAST
   fields <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::recordtype ~fqdn :wat::core::Record
       [~@fields])
     (:wat::core::defn ~fqdn [~@fields] -> ~fqdn
       (:wat::core::aggregate-new ~fqdn
         ~@(:wat::core::let
               ;; Arc 291 hygiene fix: use (ast->children (quote fields)) to get the original
               ;; AST nodes with scope preserved, not the holon round-trip (which strips scope).
               ;; The binders in [~@fields] carry the original scope (e.g. scope 433 when this
               ;; defn is emitted inside another macro's quasiquote); the body references must
               ;; carry the SAME scope — reuse the original nodes from (quote fields) directly.
               ;; (quote fields) is needed: substitute_bindings replaces `fields` with the raw
               ;; WatAST::Vector node; quote wraps it as Value::wat__WatAST for ast->children.
               ;; Arc 118.2a — was `(:wat::core::map ...)`. `map` flipped LAZY (returns a
               ;; `Stream`, not a `Vector`) and this `~@`-splice needs a concrete `Vector<WatAST>`
               ;; RIGHT NOW, at macro-expansion time — this macro is invoked from EVERY
               ;; `defrecord` call across the stdlib (~30+ sites, earliest `core.wat`'s `Fault`),
               ;; so it cannot depend on any wat-defined eager materializer either (a wat-defined
               ;; `vec`/`into` would itself be an untested/unsafe dependency at this exact
               ;; bootstrap phase — see `crate::stream::NativeLazyCell`'s doc for the full
               ;; writeup). `foldl` + `conj` stay Rust-native and eager, unaffected by the flip.
               [raw-ch  (:wat::core::ast->children (:wat::core::quote fields))
                nf      (:wat::core::i64::/ (:wat::core::length raw-ch) 3)
                syms    (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST> fi <- :wat::core::i64] -> :wat::core::Vector<wat::WatAST>
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                var-w (:wat::core::Option/expect
                                         (:wat::core::Vector/get raw-ch idx)
                                         "Record::def: struct_form field name index out of range")]
                               (:wat::core::conj acc var-w)))
                           (:wat::core::Vector :wat::WatAST)
                           (:wat::core::range 0 nf))]
               syms)))
))

;; Arc 293.R2.2 — accessor emission removed from BASE macro.
;; register_aggregate_methods (runtime.rs) now mints all field accessors for
;; every Aggregate holder (Struct + Record + HolonRecord): bare name, struct-field
;; body, type_params-aware.  The ctor defn above stays exactly as-is.

;; ─── HOLONIC macro (:wat::holon::Record::def) ────────────────────────────────

;; Arc 294.c.2a — holonic defrecord macro routes through aggregate-new (the ONE
;; holder-dispatched ctor). The entire hologram quasiquote (former lines ~157-197)
;; is DELETED — build_holon_hologram in Rust now derives the hologram internally.
;; Arc 291 hygiene preserved: raw-ch/nf/syms dance keeps scope-tagged AST nodes.
(:wat::core::defmacro :wat::holon::defrecord
  [fqdn   <- :wat::WatAST
   fields <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::recordtype ~fqdn :wat::holon::Record
       [~@fields])
     (:wat::core::defn ~fqdn [~@fields] -> ~fqdn
       (:wat::core::aggregate-new ~fqdn
         ~@(:wat::core::let
               ;; Arc 291 hygiene fix: use (ast->children (quote fields)) to get the original
               ;; AST nodes with scope preserved, not the holon round-trip (which strips scope).
               ;; The binders in [~@fields] carry the original scope; the body references must
               ;; carry the SAME scope — reuse the original nodes from (quote fields) directly.
               ;; Arc 118.2a — see the BASE macro above for why this is `foldl`+`conj`, not `map`.
               [raw-ch  (:wat::core::ast->children (:wat::core::quote fields))
                nf      (:wat::core::i64::/ (:wat::core::length raw-ch) 3)
                syms    (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST> fi <- :wat::core::i64] -> :wat::core::Vector<wat::WatAST>
                             (:wat::core::let
                               [idx   (:wat::core::i64::* fi 3)
                                var-w (:wat::core::Option/expect
                                         (:wat::core::Vector/get raw-ch idx)
                                         "Record::def: struct_form field name index out of range")]
                               (:wat::core::conj acc var-w)))
                           (:wat::core::Vector :wat::WatAST)
                           (:wat::core::range 0 nf))]
               syms)))
))

;; Arc 293.R2.2 — accessor emission removed from HOLONIC macro.
;; register_aggregate_methods (runtime.rs) now mints all field accessors for
;; every Aggregate holder (Struct + Record + HolonRecord): bare name, struct-field
;; body, type_params-aware.  The ctor defn above stays exactly as-is.
