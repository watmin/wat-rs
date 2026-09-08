;; Arc 255 / 296 — WHY the dot spelling yields Option.None. MEASURED 2026-09-08.
;;
;; It is NOT a mis-built variant. `(K {map})` is the shape of BOTH the enum map
;; ctor (296 M) and the keyword-as-accessor fall-through (234.3c, runtime.rs:3655).
;; When K does not resolve to a verb, the accessor treats K as a KEY, misses, and
;; returns None — a total lookup, never an error. The `:wat::*` blanket is what
;; lets a `:wat::core::` head reach that fall-through; a `:usr::` head is refused
;; by resolve first.
(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; the ctor — K resolves, the map is the payload
  (:wat::kernel::println (:wat::core::Option::Some {:value 7}))
  ;; the accessor — K does not resolve, so the map is the RECEIVER and K is the key
  (:wat::kernel::println (:wat::core::Option.Some {:value 7}))
  ;; ... and it finds the key when the key is actually there
  (:wat::kernel::println (:wat::core::Option.Some {:wat::core::Option.Some 42}))
  ;; a bare keyword needs no blanket at all — the accessor is not about `:wat::*`
  (:wat::kernel::println (:anything-at-all {:value 7}))
  ;; and the accessor is TOTAL: a hit is Some, a miss is None, neither is an error.
  ;; (Give the same unknown head a NON-map receiver and it is refused loudly —
  ;;  `(:wat::core::Nope 7)` raises UnknownFunction. The map receiver is the door.)
  (:wat::kernel::println (:value {:value 7})))
