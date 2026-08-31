;; Arc 209 naming-conversion stone — wat-level string helpers.
;; Loads after core.wat so all Rust string primitives are registered.
;;
;; Contents:
;;   :wat::core::string::kebab->pascal — kebab-case → PascalCase (the inverse of pascal->kebab).
;;
;; The Rust side provides the char-case floor ops this file composes:
;;   :wat::core::string::to-uppercase — minted in this stone (Piece 2).
;;   :wat::core::string::to-lowercase — minted in arc 209 C.3.
;;   :wat::core::string::split, subs, length, concat, join — existing primitives.
;;
;; Placement rubric: kebab->pascal is a wat helper (no macro needs it; it composes).
;; pascal->kebab is a Rust intrinsic (the defservice macro calls it at expand time).

;; capitalize — upcase the first char of a segment, keep the rest.
;;   "object" -> "Object", "v2" -> "V2", "" -> "".
;;
;; Arc 255 STONE "wire the wat side to wat-doc" — the door-plus-one-verb walk.
;; The `;;` prose above is unchanged; the `{...}` metadata map below is the
;; SAME facts as DATA, read by `wat_doc::from_metadata` at stdlib
;; registration (`register_stdlib_defines`, src/runtime.rs) through the same
;; shared-contract crate an intrinsic's `///` block goes through
;; (`wat_doc::parse`) — same required set, same `DocError`s. Values on the
;; closed-domain axes are ENUM SYMBOLS (`:wat::runtime::Purity::Pure`, not a
;; bare `:Pure`) so a typo is a rejected variant, not a silently-accepted
;; keyword.
(:wat::core::defn :wat::string::capitalize
  {:doc "Upcase the first character of a segment, keeping the rest unchanged."
   :added "1.0.0"
   :ret [:wat::core::String "the segment with its first character upcased"]
   :purity :wat::runtime::Purity::Pure
   :determinism :wat::runtime::Determinism::Deterministic
   :totality :wat::runtime::Totality::Total
   :expand-time :wat::runtime::ExpandTime::Legal
   :category :wat::runtime::Category::Transform
   :args [[w :wat::core::String "the segment to capitalize"]]
   ;; Arc 255 STONE "an example is a FORM, not a string" — literal wat forms,
   ;; not escaped-string source. `wat_doc::from_metadata` reads `fields[0]`/
   ;; `fields[1]` directly as the parsed forms they already are (the wat
   ;; reader that loaded THIS file produced them); nothing here is stringified.
   :examples [[(:wat::string::capitalize "object") "Object"]
              [(:wat::string::capitalize "") ""]]}
  [w <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::string::length w) 0)
    w
    (:wat::string::concat
      (:wat::string::to-uppercase (:wat::string::subs w 0 1))
      (:wat::string::subs w 1 (:wat::string::length w)))))

;; :wat::core::string::kebab->pascal — "get-object" -> "GetObject", "get" -> "Get".
;;
;; Algorithm: split on "-", capitalize each segment, join.
;; Bijection partner of :wat::core::string::pascal->kebab on the disciplined subset
;; (one uppercase letter per word, no consecutive-capital acronym runs).
(:wat::core::defn :wat::string::kebab->pascal
  [s <- :wat::core::String]
  -> :wat::core::String
  ;; Arc 118.2a — `map` flipped LAZY (returns Stream); `string::join` needs a Vector eagerly
  ;; (this string is fully materialized either way — no lazy pipeline benefit here).
  (:wat::string::join ""
    (:wat::core::mapv :wat::string::capitalize
      (:wat::string::split s "-"))))

;; :wat::core::string::strip-leading-colon — strip a leading ":" from s if present; else s unchanged.
;; ":foo-bar" → "foo-bar"; "foo-bar" → "foo-bar" (idempotent on bare strings).
;; Promoted from :wat::fix::rename-strip-colon (Arc 260.1b Part A dedup).
(:wat::core::defn :wat::string::strip-leading-colon
  [s <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::string::subs s 0 1) ":")
    (:wat::string::subs s 1 (:wat::string::length s))
    s))
