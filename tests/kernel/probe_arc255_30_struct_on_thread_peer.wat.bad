;; Stone 255.30 — a struct on a thread self-peer.
;; Pre-stone this loads: ThreadSelfPeer is the hatch, any I/O.
;; Post-stone the thread-spawn producer applies the §7 wall and refuses.
(:wat::core::defstruct :p30::S [val <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_p (:wat::test::spawn-peer (:wat::spawn::thread)
          (:wat::core::fn [self <- (:wat::kernel::Peer :- [:p30::S :wat::core::i64])]
              -> :wat::core::nil
            nil))]
    nil))
