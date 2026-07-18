;; tests/channel/wat_arc170_stone_a_drain_and_join.wat — co-located fixture (the THREAD world for T1+T3),
;; slurped via startup_beside(file!()). The Process child programs (T2/T4) are SEPARATE subprocesses and
;; live in the sibling wat_arc170_stone_a_drain_and_join_child_clean.wat / _child_panic.wat.

;; T1 — a worker thread that sends three i64s then returns nil; the parent drains via Thread/drain-and-join.
(:wat::core::defn :my::three-vals-thread
  [_rx <- :wat::kernel::Receiver<wat::core::i64>
   tx <- :wat::kernel::Sender<wat::core::i64>]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::core::Result/expect (:wat::kernel::send tx 1) "send 1 failed — receiver dropped before drain")
     _ (:wat::core::Result/expect (:wat::kernel::send tx 2) "send 2 failed — receiver dropped before drain")
     _ (:wat::core::Result/expect (:wat::kernel::send tx 3) "send 3 failed — receiver dropped before drain")]
    nil))

(:wat::core::defn :my::test::drain-thread
  [] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>
  (:wat::core::let
    [thr (:wat::kernel::spawn-thread :my::three-vals-thread)]
    (:wat::kernel::Thread/drain-and-join thr)))

;; T3 — a worker thread that panics; drain-and-join must still drain then return Err(chain).
(:wat::core::defn :my::panic-thread
  [_rx <- :wat::kernel::Receiver<wat::core::i64>
   _tx <- :wat::kernel::Sender<wat::core::i64>]
  -> :wat::core::nil
  (:wat::core::Option/expect :wat::core::None "intentional panic from stone-a thread test"))

(:wat::core::defn :my::test::drain-panicking-thread
  [] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>
  (:wat::core::let
    [thr (:wat::kernel::spawn-thread :my::panic-thread)]
    (:wat::kernel::Thread/drain-and-join thr)))

;; T2/T4 — Process/drain-and-join over an already-spawned child Process. The spawn itself
;; happens Rust-side (`build_spawn_process_call` builds the AST directly from `WatAST` nodes,
;; not a parsed Rust string, so it carries no inline-wat driver); this fn is just the join
;; call, taking the spawned Process as an argument — shared by both the clean-exit (T2) and
;; panicking (T4) child fixtures.
(:wat::core::defn :my::test::drain-process
  [proc <- :wat::kernel::Process<wat::core::nil,wat::core::nil>]
  -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>
  (:wat::kernel::Process/drain-and-join proc))
