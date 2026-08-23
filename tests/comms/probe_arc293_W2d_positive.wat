;; tests/comms/probe_arc293_W2d_positive.wat
;; Positive fixture for probe_arc293_W2d_peer_purity.rs.
;;
;; Arc 293.W.2d — positive cases that MUST type-check:
;;
;;   1. ThreadSelfPeer' carrying impure I/O type-checks (in-locus, any I/O).
;;   2. :wat::program::self-peer with pure types still type-checks (the purity gate only
;;      rejects IMPURE type args; it must not over-reject the pure case).
;;
;; Both cases must load without error — the Peer'<I,O> well-formedness gate must NOT
;; apply to ThreadSelfPeer' (it's in-locus), and must not fire on pure type args.

;; A struct type — impure (Nature::Struct).
(:wat::core::defstruct :w2d_pos::S [val <- :wat::core::i64])

;; Positive 1: spawn a thread whose self-peer is ThreadSelfPeer'<S,i64> (impure I).
;; In-locus; any I/O is allowed. The body does nothing — we only check type-checking.
(:wat::core::defn :w2d_pos::probe-thread-self-peer-impure [] -> :wat::core::nil
  (:wat::core::let
    [_peer (:wat::test::spawn-peer (:wat::spawn::thread)
              (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:w2d_pos::S :wat::core::i64])]
                  -> :wat::core::nil
                nil))]
    nil))

;; Positive 2: :wat::program::self-peer with pure types — must still type-check.
(:wat::core::defn :w2d_pos::probe-pure-wire-peer [] -> :wat::core::nil
  (:wat::core::let
    [_pair (:wat::program::self-peer :wat::core::i64 :wat::core::i64)]
    nil))
