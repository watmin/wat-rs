;; tests/types/probe_arc293_W2b_enum_purity.wat — co-located fixture (arc 293.W.2b: the purity marker)
;;
;; 293.W.2b makes enums DECLARE their purity directly via a mandatory
;; `:wat::enum::Pure` | `:wat::enum::Impure` marker on `defenum`. A `:Pure` enum
;; may hold only pure variant fields (scalars, records, other Pure enums). An
;; `:Impure` enum is unrestricted (it holds live resources, stays in its locus).
;;
;; This fixture tests Case 1: a `:wat::enum::Pure` enum whose `:Live` variant
;; declares a STRUCT field. The struct is impure (categorically — a struct may
;; hold resources and never crosses address spaces). The containment rule must
;; REJECT this at declaration time.
;;
;; GREEN after 293.W.2b: the load is REJECTED with a containment-rule error
;; naming the offending Pure enum and its impure variant field.

(:wat::core::defstruct :w2b::Conn [fd <- :wat::core::i64])              ; a struct (impure — never crosses)

(:wat::core::defenum :w2b::BadEvt :wat::enum::Pure                      ; Pure enum — containment applies
  :Idle
  :Live [c <- :w2b::Conn])                                              ; ILLEGAL: struct in a Pure enum
