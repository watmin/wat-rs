;; STONE 255.69 — a type constructs its own values, at run time too.
;;
;; `(wat.type/u8 65)` and `(wat.type/char "a")` type-checked but died
;; `#wat.runtime/UnknownFunction {:message "unknown function: :wat::type::u8"}`
;; at run time (measured, SCORE-STONE-255.68 § "The two scalar constructors"):
;; `constructor_head_key` (`src/types.rs`) redirected a `wat.type/`-spelled head
;; to its registered `:wat::core::…` intrinsic only for the seven CONTAINER
;; names (a hand-listed match arm), never for a scalar hard primitive. Fixed by
;; generalizing the SAME door (`canonical_type_key`) to every hard primitive a
;; registered intrinsic backs — no list of scalar names.
(:wat::core::defn :user::u8-dispatches [] -> wat.type/u8
  (wat.type/u8 65))

(:wat::core::defn :user::char-dispatches [] -> wat.type/char
  (wat.type/char "a"))

;; The five collection heads already resolved `wat.type/X` end-to-end before
;; this stone (SCORE-STONE-255.68); a regression guard that the generalized
;; door still reaches them through the SAME path (registry-membership guard,
;; not the old hand-listed match arm).
(:wat::core::defn :user::vector-still-dispatches [] -> (wat.type/PersistentVector :- [wat.type/i64])
  (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))
