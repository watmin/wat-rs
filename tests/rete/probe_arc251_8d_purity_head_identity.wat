;; tests/rete/probe_arc251_8d_purity_head_identity.wat — arc 251 stone 251.8d-ii (FIFTH draw).
;; Co-located fixture for the sibling probe (.rs), slurped via `call_beside_value`.
;;
;; THE DOOR UNDER TEST is `classify_expr`'s head read (`src/rete/purity.rs`, the general-list
;; arm). Every row below quotes its subject, because a QUOTE BOUNDARY is the one wat-surface
;; position a namespaced `WatAST::Symbol` survives to the purity walk: `resolve::normalize`
;; rewrites a symbol call head to its keyword everywhere else (measured — a symbol head inside
;; a `(:wat::rete::where …)` is normalized before the fence sees it, and an unresolvable one
;; raises `UnresolvedReference` at load). So an UNQUOTED row here would prove nothing about
;; the door; it would prove `normalize` works.
;;
;; ⛔ THE CURE RUNS IN THE PERMISSIVE DIRECTION — a purity gate is DEFAULT-DENY, so teaching
;; it a second spelling makes it accept MORE. The three non-vacuity families are therefore the
;; stone, not a formality:
;;   (a) a genuinely PURE verb is accepted in both spellings   — the cure
;;   (b) a genuinely IMPURE verb is refused in both spellings  — the adversarial family
;;   (c) a genuinely UNKNOWN head stays unknown in both        — "anything I can't parse is fine" is not the cure
;; plus (d) the AXES DO NOT COLLAPSE: `Uuid/v4` is pure AND non-deterministic in BOTH spellings,
;; so the door re-spells a name, it does not hand out a blanket yes.

;; ─── (a) THE CURE — a proven-pure verb, both spellings ──────────────────────────
(:wat::core::defn :user::lt-kw-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::< 1 2))))
(:wat::core::defn :user::lt-sym-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (wat.core/< 1 2))))

;; ─── (b) THE ADVERSARIAL FAMILY — a genuinely IMPURE verb, both spellings ───────
;; `:wat::kernel::` is one of `effectful_by_prefix`'s prefixes. Pre-cure the symbol spelling
;; was refused only because it fell off the END of the table (unknown); post-cure it is
;; refused BY THE PREFIX, which is the reason that was meant all along.
(:wat::core::defn :user::println-kw-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::kernel::println "x"))))
(:wat::core::defn :user::println-sym-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (wat.kernel/println "x"))))
;; A second impure family (`:wat::io::`), so the row is not one prefix wide.
(:wat::core::defn :user::io-kw-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::io::read-file "x"))))
(:wat::core::defn :user::io-sym-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (wat.io/read-file "x"))))

;; ─── (c) AN UNKNOWN HEAD STAYS UNKNOWN, both spellings ──────────────────────────
(:wat::core::defn :user::unknown-kw-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:not::a::real::op 1))))
(:wat::core::defn :user::unknown-sym-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (not.a.real/op 1))))
;; A BARE symbol head names nothing and must stay unknown — the door leaves a symbol with no
;; namespace byte-identical, so this is also the proof it did not start minting identities.
(:wat::core::defn :user::bare-sym-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (nope 1))))

;; ─── (d) THE AXES DO NOT COLLAPSE — Uuid/v4 is pure ∧ NON-deterministic ─────────
(:wat::core::defn :user::uuid-kw-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::uuid::v4))))
(:wat::core::defn :user::uuid-sym-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (wat.uuid/v4))))
(:wat::core::defn :user::uuid-kw-det? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:wat::uuid::v4))))
(:wat::core::defn :user::uuid-sym-det? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (wat.uuid/v4))))

;; ─── (e) LAW A IS NOT LOOSENED — a CORE-spelled head is still not a rete primitive ──
;; The permissive direction's sharpest edge: the door must not turn `wat.core/<` into an
;; admissible `where` operand. Law A refuses a core-spelled op whatever its spelling.
(:wat::core::defn :user::lt-kw-primitive? [] -> :wat::core::bool
  (:wat::rete::primitive? (:wat::core::quote (:wat::core::< 1 2))))
(:wat::core::defn :user::lt-sym-primitive? [] -> :wat::core::bool
  (:wat::rete::primitive? (:wat::core::quote (wat.core/< 1 2))))
;; …while a genuinely RETE-namespaced head IS admitted in both spellings — the door's whole
;; job. Without this row the two above would pass on a gate that refused everything.
(:wat::core::defn :user::rete-kw-primitive? [] -> :wat::core::bool
  (:wat::rete::primitive? (:wat::core::quote (:wat::rete::i64::< 1 2))))
(:wat::core::defn :user::rete-sym-primitive? [] -> :wat::core::bool
  (:wat::rete::primitive? (:wat::core::quote (wat.rete.i64/< 1 2))))
