;; STONE 255.70 — a constructor's `:-` bracket reads through the type door; `List` takes `:-`.
;;
;; SCORE-STONE-255.69 found two checker gaps blocking the last 109 (35 + 74) sites:
;;
;;   1. `parse_bracket_type_keyword` (`src/check.rs`) accepted only a bare `WatAST::Keyword` per
;;      slot of a constructor's own `:-` bracket, so a COMPOUND element type — a nested
;;      `(Head :- […])` form, e.g. a vector of tuples or a map of string to vector — failed to
;;      check: `#wat.check/MalformedForm {:reason "bracketed type must be a type keyword"}`.
;;   2. `infer_linked_list_constructor` never learned the `:-` binder at all, so
;;      `(wat.type/List :- [wat.type/i64] 1 2 3)` failed to check.
;;
;; Both are fixed this stone: every constructor bracket slot now reads through
;; `parse_param_spec_slot` → `parse_type_node`, the one door (K1) that already parses a bare
;; keyword, a namespaced symbol, a nested parametric form, and a fn-type bracket identically; and
;; `List` now peels an OPTIONAL `:- [T]` bracket exactly as `PersistentVector` does (declared T is
;; the up-cast target; bracket-less stays a fresh unified variable — the wall making the bracket
;; MANDATORY is a later stone).

;; ── Compound element type: a PersistentVector of Tuples ──────────────────────────────────────
(:wat::core::defn :user::vector-of-tuples [] -> (wat.type/PersistentVector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64])])
  (wat.type/PersistentVector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64])]
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 2)
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 3 4)))

;; ── Compound element type: a PersistentMap of String to PersistentVector ─────────────────────
(:wat::core::defn :user::map-string-to-vector [] -> (wat.type/PersistentMap :- [wat.type/String (wat.type/PersistentVector :- [wat.type/i64])])
  (wat.type/PersistentMap :- [wat.type/String (wat.type/PersistentVector :- [wat.type/i64])]
    "a" (wat.type/PersistentVector :- [wat.type/i64] 1 2)
    "b" (wat.type/PersistentVector :- [wat.type/i64] 3 4)))

;; ── `List` takes `:-` — value position, non-empty ────────────────────────────────────────────
(:wat::core::defn :user::list-typed [] -> (wat.type/List :- [wat.type/i64])
  (wat.type/List :- [wat.type/i64] 1 2 3))

;; ── `List` takes `:-` — the empty bracketed list is a legitimate empty value ─────────────────
(:wat::core::defn :user::list-typed-empty [] -> (wat.type/List :- [wat.type/i64])
  (wat.type/List :- [wat.type/i64]))

;; `:user::list-bracketless-still-runs` (a bracket-less `(wat.type/List 1 2 3)`, the STOP-2
;; regression guard this stone required — "the bracket stays optional") MOVED OUT arc 255 STONE
;; 71 (THE WALL): a bracket-less `List` call is illegal now, and its presence here would break
;; this whole file's freeze (Rust's `call_beside_value` panics on a freeze error, taking down
;; every OTHER row in this file with it). Its replacement — proving the WALL refuses it, by
;; name — lives at `tests/function/probe_stone255_71_list_bracketless_illegal.wat.bad` /
;; `.rs`.

;; ── `List` with a COMPOUND element type — both fixes together ────────────────────────────────
(:wat::core::defn :user::list-of-tuples [] -> (wat.type/List :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64])])
  (wat.type/List :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64])]
    (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 2)))
