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
(:wat::core::defn :wat::core::string::capitalize
  [w <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::string::length w) 0)
    w
    (:wat::core::string::concat
      (:wat::core::string::to-uppercase (:wat::core::string::subs w 0 1))
      (:wat::core::string::subs w 1 (:wat::core::string::length w)))))

;; :wat::core::string::kebab->pascal — "get-object" -> "GetObject", "get" -> "Get".
;;
;; Algorithm: split on "-", capitalize each segment, join.
;; Bijection partner of :wat::core::string::pascal->kebab on the disciplined subset
;; (one uppercase letter per word, no consecutive-capital acronym runs).
(:wat::core::defn :wat::core::string::kebab->pascal
  [s <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::string::join ""
    (:wat::core::map :wat::core::string::capitalize
      (:wat::core::string::split s "-"))))
