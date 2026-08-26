;; tests/types/probe_stone_binder_is_universal.wat — co-located fixture for
;; probe_stone_binder_is_universal.rs (STONE-the-binder-must-be-universal, arc 109).
;;
;; Ten root-level substrate eval forms (:wat::eval-ast! / eval-with-defs! / eval-step! /
;; eval::walk / eval-edn! / eval-file! / eval-digest! / eval-digest-string! / eval-signed! /
;; eval-signed-string!) never learned arc 109's call-site `:- […]` binder: the CHECKER
;; accepted it, the runtime dispatch cluster (src/runtime.rs, the ten `":wat::eval-…!" =>`
;; arms) refused it as an extra argument. Fixed by peeling the binder ONCE at the dispatch
;; cluster, before the family's ten arms.
;;
;; No :user::main needed — startup_beside loads defns; the sibling .rs calls each fn via
;; apply_function against the frozen world (the `wat_eval_result.wat` idiom).

;; ─── Row 1 — generic form, NON-empty binder. Load-bearing: this is the one that failed
;; before the fix ("takes exactly 1 argument; got 3"). `:wat::eval-ast!` is registered
;; `∀T. WatAST -> Result<T, EvalError>`; the caller binds T = i64 explicitly. ──────────

(:wat::core::defn :t::row1_generic_binder [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::EvalError])
  (:wat::core::let
    [program (:wat::core::quote (:wat::i64::+ 40 2))]
    (:wat::eval-ast! :- [:wat::core::i64] program)))

;; ─── Rows 2 / 3 — a NON-generic form (`:wat::eval-edn!`, `type_params: vec![]`) with an
;; EMPTY binder vs. no binder at all. `:- []` peels to `Some(&[])`, never `None` — arc
;; 109's own ruling is that the empty binder is *absent*, not merely harmless. Both must
;; behave identically. ─────────────────────────────────────────────────────────────────

(:wat::core::defn :t::row2_empty_binder [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-edn! :- [] "42"))

(:wat::core::defn :t::row3_no_binder [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-edn! "42"))

;; ─── Row 4 — structural, not one-armed: repeat the row-2 shape (non-generic, empty
;; binder ≡ absent) on a DIFFERENT form — one of the five the "takes exactly N argument"
;; message-grep cannot see (it counts via a shared `eval_form_digest_shared` helper, whose
;; message the per-verb function body never spells out). Proves the peel at the dispatch
;; cluster covers a form reached through a shared helper too, not just the five with an
;; inline arity message. ──────────────────────────────────────────────────────────────

(:wat::core::defn :t::row4_empty_binder_second_form [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-digest-string!
    :- []
    "(:wat::holon::to-holon \"x\")"
    :wat::verify::digest-sha256
    :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"))

(:wat::core::defn :t::row4_no_binder_second_form [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-digest-string!
    "(:wat::holon::to-holon \"x\")"
    :wat::verify::digest-sha256
    :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"))
