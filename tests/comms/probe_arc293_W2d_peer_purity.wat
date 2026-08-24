;; tests/comms/probe_arc293_W2d_peer_purity.wat
;; Co-located RED probe for probe_arc293_W2d_peer_purity.rs (startup_beside).
;;
;; Arc 293.W.2d — compile-time wire-wall: (Peer' :- [I O]) well-formedness.
;;
;; After 2d: `:wat::program::self-peer` with a struct type arg is rejected at CHECK —
;; a wire peer's I and O must be `:Pure` types.
;;
;; RED at HEAD: this loads cleanly (no purity check on Peer' producers yet).
;; GREEN after 2d: the :wat::program::self-peer purity gate fires, world fails to load.

(:wat::core::defstruct :w2d::S [val <- :wat::core::i64])

;; Creates (Peer' :- [:w2d::S :wat::core::i64]) — struct type arg is impure.
;; After 2d: the :wat::program::self-peer producer checks is_pure_type(:w2d::S) → false → CHECK error.
(:wat::core::defn :w2d::probe-impure-wire-peer [] -> :wat::core::nil
  (:wat::core::let
    [_pair (:wat::program::self-peer :w2d::S :wat::core::i64)]
    nil))
