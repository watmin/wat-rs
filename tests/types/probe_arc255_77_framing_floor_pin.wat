;; Arc 255.77 — pins `:wat::telemetry::framing-floor-of`'s numbers for each of the four
;; FIXED-VALUE branches (i64/f64/Uuid/bool) BEFORE the UUID type-key rename
;; (`:wat::core::Uuid` -> `wat.uuid/UUID`) and before the trap's cure (the branch compared
;; `(:wat::core::ast-name t)` against a string like `"wat.type/Uuid"` — renamer-fragile; the
;; cure compares the field's type node AS DATA via `:wat::core::type-equal?`, through the
;; type door, for ALL FOUR branches, not just Uuid).
;;
;; Each record below has exactly ONE field named `:v`, isolating that branch's fixed-cost
;; contribution from the shared key-cost. This file is one of the codemod's own corpus targets
;; (`wat-scripts/fixes/uuid-type-goes-home.wat`) — its `:wat::core::Uuid` field type is rewritten
;; to `wat.uuid/UUID` in the SAME commit as the Rust-side rename, so this fixture exercises the
;; post-rename spelling after conversion while this probe (`.rs` companion) keeps asserting the
;; SAME four pinned numbers — proving the rename + the trap's cure are both numerically inert.
(wat.core/defrecord probe/FFFloorI64  [v :- wat.type/i64])
(wat.core/defrecord probe/FFFloorF64  [v :- wat.type/f64])
(wat.core/defrecord probe/FFFloorUuid [v :- wat.uuid/UUID])
(wat.core/defrecord probe/FFFloorBool [v :- wat.type/bool])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/do
    (wat.kernel/println (wat.telemetry/framing-floor-of probe/FFFloorI64))
    (wat.kernel/println (wat.telemetry/framing-floor-of probe/FFFloorF64))
    (wat.kernel/println (wat.telemetry/framing-floor-of probe/FFFloorUuid))
    (wat.kernel/println (wat.telemetry/framing-floor-of probe/FFFloorBool))))
