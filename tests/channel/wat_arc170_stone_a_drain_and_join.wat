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
  [] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::LociDiedError>>
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
  [] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::LociDiedError>>
  (:wat::core::let
    [thr (:wat::kernel::spawn-thread :my::panic-thread)]
    (:wat::kernel::Thread/drain-and-join thr)))

;; T2/T4 — recv-all' drain over an already-spawned process peer (arc 278 IPC de-prime:
;; the drain-then-join maps to the honest peer-drain recv-all'). The spawn itself happens
;; Rust-side (`build_spawn_program_process_call` builds the `spawn-program' (process)` AST
;; directly from `WatAST` nodes, not a parsed Rust string, so it carries no inline-wat
;; driver); this fn is just the drain call, taking the spawned Process' peer as an argument —
;; shared by both the clean-exit (T2, prints Strings then Closes) and panicking (T4, dies →
;; Lost) child fixtures. recv-all' returns Ok(collected outputs) on a clean Closed, or
;; Err(cause) when the peer DIED — the LociDiedError rides in the Err, surfaced, never swallowed.
(:wat::core::defn :my::test::drain-process
  [p <- :wat::kernel::Peer'<wat::core::nil,wat::core::String>]
  -> :wat::core::Result<wat::core::Vector<wat::core::String>,wat::kernel::LociDiedError>
  (:wat::kernel::recv-all' p))
