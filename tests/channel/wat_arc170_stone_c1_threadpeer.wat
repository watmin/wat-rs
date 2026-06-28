;; tests/channel/wat_arc170_stone_c1_threadpeer.wat — co-located fixture for T1 (the type mint),
;; slurped via startup_beside(file!()). Both ThreadPeer orientations as fn param types; the probe asserts
;; both defns are present after freeze (the parametric type resolves). They are never called.

(:wat::core::defn :my::server-side
  [_peer <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::String>]
  -> :wat::core::nil
  nil)

(:wat::core::defn :my::client-side
  [_peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>]
  -> :wat::core::nil
  nil)
