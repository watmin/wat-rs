;; Arc 255.77 — `wat.uuid/UUID` (the Uuid type's new home spelling, post cutover stone 3) checks
;; and runs in a HEADER (fn param + return type) and as a SET ELEMENT (`HashSet<wat.uuid/UUID>`,
;; which requires Equatable + Hashable for membership dedup). Record-field and map-key positions,
;; plus the Equatable `extend-type`, are already covered by the corpus's own converted sites
;; (`tests/types/probe_arc241_stone8_defstruct_c04.wat`'s record fields,
;; `tests/value/wat_arc221_keyword_nil_tag_atomization.wat`'s `HashMap<Uuid,String>`,
;; `wat/class.wat`'s `(extend-type wat.uuid/UUID :wat::core::Equatable)`); the retirement
;; refusal is covered end-to-end by `tests/cli/retirement_table_reachable.rs` (walks
;; `RETIREMENT_TABLE` itself, so it picks up this stone's new row automatically).
;;
;; Freeze-time assertions (top-level `def`s evaluate eagerly) — `world.is_ok()` on the Rust side
;; is the single honest gate, same pattern as `probe_arc278_capacity_derive.wat` beside this one.

;; HEADER — a fn whose param type AND return type are both `wat.uuid/UUID`.
(wat.core/defn test/identity-uuid [u :- wat.uuid/UUID] :- wat.uuid/UUID
  u)

(wat.core/def probe/u1 (wat.uuid/v4))
(wat.core/def probe/u2 (test/identity-uuid probe/u1))

;; SET ELEMENT — `HashSet<wat.uuid/UUID>`; u1 and u2 are the SAME uuid (round-tripped through
;; identity-uuid), so a correctly-Equatable/Hashable set collapses them to one member.
(wat.core/def probe/uuid-set
  (wat.type/HashSet :- [wat.uuid/UUID] probe/u1 probe/u2))

(wat.core/def probe/assert-header-roundtrip
  (wat.test/assert-true (wat.core/= probe/u1 probe/u2)))
(wat.core/def probe/assert-set-dedup
  (wat.test/assert-true (wat.core/= (wat.core/length probe/uuid-set) 1)))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println "arc255.77: wat.uuid/UUID header + set element proven"))
