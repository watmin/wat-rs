;; Stone 118.B2c strike 1 — WITNESS: two `defclause` arms with the IDENTICAL declared type and
;; DIFFERENT bodies are accepted silently, and the second body is DEAD CODE.
;;
;; This fixture LOADS and TYPE-CHECKS cleanly. That is the defect.
;;
;; ★ IT IS A HOLE IN THE REDEF RULE. Arc 054 (`docs/arc/2026/04/054-idempotent-redeclare/`) made
;; `typealias` / `define` / `defmacro` "if byte-equivalent, no-op", else `DuplicateDefine`; arc 157
;; added the opt-in `redef_allowed` with a type-stability check. `defclause` ARMS were never covered,
;; because an arm is not a definition BY NAME — so the one registry that dispatches on TYPES is the
;; one registry with no define-once rule.
;;
;; Dispatch is first-match-wins in declaration order (`src/runtime.rs`, the
;; `for (clause_idx, clause) in cs.clauses.iter().enumerate()` loop returns on the first match, with
;; no most-specific selection). So `:my::pick` answers "FIRST" and nothing ever reaches "SECOND".
;;
;; Builder, 2026-08-18: *"you may only express something's def once and all other attempts must be
;; identical."* Strike 1 makes that true for clause arms.

(:wat::core::defclause :my::pick
  ([x <- :wat::core::i64] -> :wat::core::String "FIRST")
  ([x <- :wat::core::i64] -> :wat::core::String "SECOND"))

(:wat::core::defn :my::which [] -> :wat::core::String
  (:my::pick 1))

;; ─── the NON-VACUITY control: a normal multi-arm defclause (DISJOINT types) must keep working ───
;; Without this, "overlapping arms are refused" would be satisfied by a substrate that refused ALL
;; multi-arm defclauses — which would take the entire language with it.

(:wat::core::defclause :my::describe
  ([x <- :wat::core::i64]         -> :wat::core::String "an int")
  ([x <- :wat::core::String]      -> :wat::core::String "a string")
  ([x <- :wat::core::bool]        -> :wat::core::String "a bool"))

(:wat::core::defn :my::describe-int    [] -> :wat::core::String (:my::describe 1))
(:wat::core::defn :my::describe-string [] -> :wat::core::String (:my::describe "s"))
(:wat::core::defn :my::describe-bool   [] -> :wat::core::String (:my::describe true))
