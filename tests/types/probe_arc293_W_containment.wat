;; tests/types/probe_arc293_W_containment.wat — co-located fixture (arc 293.W, the deep wire wall)
;;
;; THE CONTAINMENT RULE: a portable aggregate (record/holon) may hold ONLY portable fields. A `Struct`
;; field is ILLEGAL at declaration — a struct cannot be reconstructed from EDN bytes on the far side
;; (no default for a bound resource), so a record that held one could never cross, so it must not exist.
;; This turns §7's "a struct crosses NO comms" into a TYPE guarantee (a record cannot HOLD a struct →
;; can never CARRY one across — the wire-wall breach becomes unrepresentable).
;;
;; This fixture is INTENTIONALLY ILLEGAL: it declares a record with a struct field. It must FAIL to load.
;; RED at HEAD: it loads cleanly (the grounded breach — a struct then crosses a process peer). GREEN after
;; 293.W: the load is REJECTED with a containment-rule error naming the offending field.

(:wat::core::defstruct :w::Conn [fd <- :wat::core::i64])                      ; a struct (in-locus, non-portable)

(:wat::core::defrecord :w::Bad  [tag <- :wat::core::i64  c <- :w::Conn])      ; ILLEGAL — a record cannot hold a struct
