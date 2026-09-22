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
;;
;; Arc 294 item 9a — CONSTRUCTION ERGONOMICS FLIP. The bare type name `~fqdn` is now the
;; KWARGS macro (order-free `(:ns::T :field 1 …)`); the raw positional ctor moved to the
;; PRIME `:ns::T'` (minted in `register_aggregate_methods`, runtime.rs). This macro now
;; emits a `do` of TWO forms: the `recordtype` decl (unchanged) + a companion `defmacro`
;; at the bare `~fqdn` that thin-forwards to `:wat::core::kwargs-lower` in its pure-
;; positional mode (sentinel `:wat::core::agg-positional`) — mirrors `:wat::core::defn`'s
;; own companion-emission template (wat/core.wat, the kwargs branch) at a shallower depth
;; (no $impl fn / Coords / grant-handles — just the reorder-and-call).
;;
;; The companion QUASIQUOTES the kwargs-lower call (returns the form; does not evaluate
;; kwargs-lower itself) — a data-skip past the F5 purity gate, proven by the arc-294
;; de-risk (`derisk_agg_kwargs.wat`).
(:wat::core::defmacro :wat::core::defrecord
  [& args <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  ;; Arc 293 surface-splice — the constructor `defn` is DELETED from this macro. The ctor
  ;; is now minted (for EVERY aggregate nature) in `register_aggregate_methods` (runtime.rs)
  ;; from the REGISTERED fields — so `~@:Surface` splices in the field vector are expanded
  ;; (at type registration) BEFORE the ctor is built, and records get splice for free.
  ;; The old expand-time `raw-ch/nf/syms` groups-of-3 walk (registry-blind, choked on `~@`)
  ;; is gone along with the whole lazy-seq / arc-118 concern.
  ;;
  ;; NOTE: the kwargs-companion field-name extraction below walks `fields` at THIS macro's
  ;; own expansion time — BEFORE `~@:Surface` splices are resolved (that happens later, at
  ;; type-registration, in `parse_aggregate_fields_with_splices`). A splice-bearing field
  ;; vector (e.g. wat/telemetry.wat's `Metric`/`Log`) therefore has its `~@:Surface` elements
  ;; SKIPPED here (never `ast-name`d — they are not field names) — the companion bakes only
  ;; the record's OWN literal `name <- :T` fields; the full post-splice field list (own +
  ;; spliced) is minted later at type-registration for the prime ctor + accessors. Mirrors
  ;; the splice-node shape check in `parse_aggregate_fields_with_splices` / `splice_target`
  ;; (src/types/defstruct.rs:326 — `(:wat::core::unquote-splicing :Surface)`).
  ;;
  ;; Arc 109 β-i — the declaration binder. Fully variadic now, mirroring `:wat::core::defstruct`
  ;; (wat/core.wat): slots are picked from the ENDS, never by counting. `fqdn` is the first
  ;; arg, `fields` is the last; the optional `:- [T…]` binder pair (if present) is whatever
  ;; sits between them — peeled off by dropping the front (`rest`) and dropping the back
  ;; (`reverse`+`rest`+`reverse`), so it comes out `[]` for the ordinary
  ;; `(defrecord :Name [fields])` call and `[:- [T…]]` for `(defrecord :Name :- [T…] [fields])`.
  ;; No arity check anywhere in this body — that is the whole point of the stone.
  (:wat::core::let
    [fqdn         (:wat::core::first args)
     ;; ⚠ Arc 109 β-i — `tail`/`Option::expect` was TRIED here for a friendlier missing-field
     ;; message and REVERTED: `Option/expect` in a macro body PANICS (a `#wat.kernel/AssertionFailure`
     ;; that aborts the thread) rather than producing a structured macro error, so
     ;; `expect_startup_err` gets nothing to inspect. The shape below keeps the error a real
     ;; VALUE — `ProgramBodyEvalFailed` wrapping the failing primitive — which is what every
     ;; consumer of a macro error can actually read. See
     ;; NOTE-a-macro-cannot-diagnose-with-option-expect.md.
     fields       (:wat::core::Option/expect (:wat::core::last args) "defrecord: missing field-vector")
     binder       (:wat::core::reverse (:wat::core::rest (:wat::core::reverse (:wat::core::rest args))))
     field-ch     (:wat::core::ast->children fields)
     clean-field-ch
     (:wat::core::foldl
       (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
         (:wat::core::let
           [item      (:wat::core::Option/expect (:wat::core::get field-ch i) "defrecord kwargs companion: field-ch index")
            is-splice (:wat::core::if (:wat::core::= (:wat::core::ast-kind item) "list")
                        (:wat::core::= (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children item))) ":wat::core::unquote-splicing")
                        false)]
           (:wat::core::if is-splice
             acc
             (:wat::core::conj acc item))))
       (:wat::core::Vector :- [:wat::WatAST])
       (:wat::core::range 0 (:wat::core::length field-ch)))
     field-len    (:wat::core::length clean-field-ch)
     n-fields     (:wat::i64::/ field-len 3)
     fname-nodes  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                      (:wat::core::conj acc
                        (:wat::core::Option/expect
                          (:wat::core::get clean-field-ch (:wat::i64::* i 3))
                          "defrecord kwargs companion: fname index")))
                    (:wat::core::Vector :- [:wat::WatAST])
                    (:wat::core::range 0 n-fields))
     field-names-ast-vec (:wat::core::with-children fields fname-nodes)
     fqdn-str      (:wat::keyword::to-string fqdn)
     ;; Arc 294 item 9a — a GENERIC type name registers its kwargs companion + references
     ;; its positional prime under the BARE name (params ride ONLY on the recordtype decl,
     ;; `~fqdn` below). Matches register_aggregate_methods (`format!("{}'", agg.name)`).
     fqdn-bare-str (:wat::core::first (:wat::string::split fqdn-str "<"))
     fqdn-bare-kw  (:wat::core::keyword-node (:wat::string::interpolate ":{fqdn-bare-str}" :fqdn-bare-str fqdn-bare-str))
     ;; Arc 294 item (C) — the bare `:T` keyword STRING, spliced into the companion's
     ;; live `kwargs-construct` form (check/eval read the field order off the registry).
     bare-kw-str   (:wat::string::interpolate ":{fqdn-bare-str}" :fqdn-bare-str fqdn-bare-str)
     prime-kw-str  (:wat::string::concat ":" (:wat::string::concat fqdn-bare-str "'"))
     ns-parts      (:wat::string::split fqdn-bare-str "::")
     n-ns-parts    (:wat::core::length ns-parts)
     ns-lead       (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
                       (:wat::core::conj acc
                         (:wat::core::Option/expect (:wat::core::get ns-parts i) "defrecord kwargs companion: ns-part index")))
                     (:wat::core::Vector :- [:wat::core::String])
                     (:wat::core::range 0 (:wat::i64::- n-ns-parts 1)))
     ns-joined     (:wat::string::join "::" ns-lead)
     ns-colon-str  (:wat::string::concat ":" (:wat::string::concat ns-joined "::"))
     call-args-sym (:wat::core::symbol-node "call-args")]
    ;; The macro now expands to `(do recordtype companion)` — NOT emptied by `register_types`
    ;; (which strips only the type decl from a `do` body): the companion `defmacro` survives.
    `(:wat::core::do
       (:wat::core::recordtype ~fqdn ~@binder :wat::core::Record
         [~@field-ch])
       (:wat::core::defmacro ~fqdn-bare-kw
         [& ~call-args-sym <- (:wat::core::Vector :- [:wat::WatAST])]
         -> :wat::WatAST
         ;; Arc 294 item (C) — emit the LIVE `kwargs-construct` form over the bare `:T`
         ;; keyword; check/eval resolve `:T`'s (splice-merged, post-register) field order
         ;; and reorder the kwargs there. Replaces the expand-time `kwargs-lower` forward,
         ;; whose baked field-vector is WRONG for a SPLICED record (the splice isn't
         ;; resolved until `register_types`).
         (:wat::core::let
           [~(:wat::core::symbol-node "_kc-type") (:wat::core::keyword-node ~bare-kw-str)]
           `(:wat::core::kwargs-construct ~_kc-type ~@call-args))))))

;; Arc 293.R2.2 — accessor emission removed from BASE macro.
;; register_aggregate_methods (runtime.rs) now mints all field accessors for
;; every Aggregate nature (Struct + Record + HolonRecord): bare name, struct-field
;; body, type_params-aware.  The ctor defn above stays exactly as-is.

;; ─── HOLONIC macro (:wat::holon::Record::def) ────────────────────────────────

;; Arc 294.c.2a — holonic defrecord macro routes through aggregate-new (the ONE
;; nature-dispatched ctor). The entire hologram quasiquote (former lines ~157-197)
;; is DELETED — build_holon_hologram in Rust now derives the hologram internally.
;; Arc 291 hygiene preserved: raw-ch/nf/syms dance keeps scope-tagged AST nodes.
;;
;; Arc 294 item 9a — CONSTRUCTION ERGONOMICS FLIP (same shape as the BASE macro above;
;; only the `recordtype` parent differs: `:wat::holon::Record` vs `:wat::core::Record`).
;; See the BASE macro's comments for the full rationale + the splice-field known gap.
(:wat::core::defmacro :wat::holon::defrecord
  [& args <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  ;; Arc 293 surface-splice — constructor `defn` DELETED (see the BASE macro above). The
  ;; holon ctor is minted in `register_aggregate_methods` from the registered fields; the
  ;; `aggregate-new` body is nature-blind and derives the hologram internally for HolonRecord.
  ;;
  ;; Arc 109 β-i — same variadic/ends-only binder shape as the BASE macro above; see there
  ;; for the full rationale. `:wat::holon::defrecord` has zero parametric call sites in the
  ;; corpus today; the `:- [T…]`-binder is the only spelling the lexer admits (arc 109 —
  ;; angle-bracket type parameters are illegal in a name).
  (:wat::core::let
    [fqdn         (:wat::core::first args)
     fields       (:wat::core::Option/expect (:wat::core::last args) "holon defrecord: missing field-vector")
     binder       (:wat::core::reverse (:wat::core::rest (:wat::core::reverse (:wat::core::rest args))))
     field-ch     (:wat::core::ast->children fields)
     ;; Arc 294 item 9a fix (surface-splice regression) — see the BASE macro above for the
     ;; full rationale: skip `~@:Surface` splice elements before the name/<-/type triple
     ;; walk, so the companion bakes only the record's OWN literal fields.
     clean-field-ch
     (:wat::core::foldl
       (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
         (:wat::core::let
           [item      (:wat::core::Option/expect (:wat::core::get field-ch i) "holon defrecord kwargs companion: field-ch index")
            is-splice (:wat::core::if (:wat::core::= (:wat::core::ast-kind item) "list")
                        (:wat::core::= (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children item))) ":wat::core::unquote-splicing")
                        false)]
           (:wat::core::if is-splice
             acc
             (:wat::core::conj acc item))))
       (:wat::core::Vector :- [:wat::WatAST])
       (:wat::core::range 0 (:wat::core::length field-ch)))
     field-len    (:wat::core::length clean-field-ch)
     n-fields     (:wat::i64::/ field-len 3)
     fname-nodes  (:wat::core::foldl
                    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
                      (:wat::core::conj acc
                        (:wat::core::Option/expect
                          (:wat::core::get clean-field-ch (:wat::i64::* i 3))
                          "holon defrecord kwargs companion: fname index")))
                    (:wat::core::Vector :- [:wat::WatAST])
                    (:wat::core::range 0 n-fields))
     field-names-ast-vec (:wat::core::with-children fields fname-nodes)
     fqdn-str      (:wat::keyword::to-string fqdn)
     ;; Arc 294 item 9a — a GENERIC type name registers its kwargs companion + references
     ;; its positional prime under the BARE name (params ride ONLY on the recordtype decl,
     ;; `~fqdn` below). Matches register_aggregate_methods (`format!("{}'", agg.name)`).
     fqdn-bare-str (:wat::core::first (:wat::string::split fqdn-str "<"))
     fqdn-bare-kw  (:wat::core::keyword-node (:wat::string::interpolate ":{fqdn-bare-str}" :fqdn-bare-str fqdn-bare-str))
     ;; Arc 294 item (C) — the bare `:T` keyword STRING for the live `kwargs-construct`.
     bare-kw-str   (:wat::string::interpolate ":{fqdn-bare-str}" :fqdn-bare-str fqdn-bare-str)
     prime-kw-str  (:wat::string::concat ":" (:wat::string::concat fqdn-bare-str "'"))
     ns-parts      (:wat::string::split fqdn-bare-str "::")
     n-ns-parts    (:wat::core::length ns-parts)
     ns-lead       (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
                       (:wat::core::conj acc
                         (:wat::core::Option/expect (:wat::core::get ns-parts i) "holon defrecord kwargs companion: ns-part index")))
                     (:wat::core::Vector :- [:wat::core::String])
                     (:wat::core::range 0 (:wat::i64::- n-ns-parts 1)))
     ns-joined     (:wat::string::join "::" ns-lead)
     ns-colon-str  (:wat::string::concat ":" (:wat::string::concat ns-joined "::"))
     call-args-sym (:wat::core::symbol-node "call-args")]
    `(:wat::core::do
       (:wat::core::recordtype ~fqdn ~@binder :wat::holon::Record
         [~@field-ch])
       (:wat::core::defmacro ~fqdn-bare-kw
         [& ~call-args-sym <- (:wat::core::Vector :- [:wat::WatAST])]
         -> :wat::WatAST
         ;; Arc 294 item (C) — LIVE `kwargs-construct` over the bare `:T` (see the BASE macro).
         (:wat::core::let
           [~(:wat::core::symbol-node "_kc-type") (:wat::core::keyword-node ~bare-kw-str)]
           `(:wat::core::kwargs-construct ~_kc-type ~@call-args))))))

;; Arc 293.R2.2 — accessor emission removed from HOLONIC macro.
;; register_aggregate_methods (runtime.rs) now mints all field accessors for
;; every Aggregate nature (Struct + Record + HolonRecord): bare name, struct-field
;; body, type_params-aware.  The ctor defn above stays exactly as-is.
