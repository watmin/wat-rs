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

;; T2/T3 — verb dispatch + type-param swap (just-eval rubric, docs/CONVENTIONS.md § Test
;; idioms): peer_a/peer_b are Rust-native handles (impure, non-EDN) minted by
;; `make_thread_peer_pair_for_test` — the `.rs` driver `apply_function`s these fns with the
;; peer as an argument, one fn per literal call the inline drivers used to make.

(:wat::core::defn :my::write-i64-42
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>]
  -> :wat::core::nil
  (:wat::kernel::Thread/println peer 42))

(:wat::core::defn :my::write-i64-7
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>]
  -> :wat::core::nil
  (:wat::kernel::Thread/println peer 7))

(:wat::core::defn :my::write-pong
  [peer <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::String>]
  -> :wat::core::nil
  (:wat::kernel::Thread/println peer "pong"))

(:wat::core::defn :my::read-i64
  [peer <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::String>]
  -> :wat::core::i64
  (:wat::kernel::Thread/readln peer))

(:wat::core::defn :my::read-string
  [peer <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>]
  -> :wat::core::String
  (:wat::kernel::Thread/readln peer))
