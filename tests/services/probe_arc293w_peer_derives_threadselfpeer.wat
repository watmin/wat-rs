;; probe_arc293w_peer_derives_threadselfpeer.wat — the SAFE direction of the peer relation.
;;
;; Arc 293.W.2d minted two peer heads on the SHARED-MEMORY-OR-NOT line:
;;   (ThreadSelfPeer' :- [S R]) — in-locus, ANY I/O (the escape hatch for peers holding live handles)
;;   (Peer' :- [S R])           — wire-safe, PURE I/O only
;;
;; `Peer'` is STRICTLY STRICTER, so a `Peer'` satisfies every constraint a `ThreadSelfPeer'`
;; position imposes. This file asserts that safe direction: a `Peer'`-typed value is accepted
;; where a `ThreadSelfPeer'` is expected. Its `.wat.bad` sibling asserts the REVERSE is still
;; refused — that one is the mobility wall and must never pass.
;;
;; Type args are IDENTICAL on both sides here on purpose: the relation is a HEAD edge, and the
;; args stay INVARIANT (a channel's send/recv types are exact — check.rs `assignable`'s
;; Parametric<:Parametric arm unifies them).

(:wat::core::defn :probe::takes-thread-self-peer
  [p <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64
  1)

;; ★ THE SUBJECT — a `Peer'` handed to a `ThreadSelfPeer'` parameter. Before the derive edge
;; this is a located TypeMismatch; after it, it type-checks by the derive graph.
(:wat::core::defn :probe::peer-satisfies-thread-self-peer
  [p <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64
  (:probe::takes-thread-self-peer p))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "peer-satisfies-thread-self-peer: checked"))
