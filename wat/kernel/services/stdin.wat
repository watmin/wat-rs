;; wat/kernel/services/stdin.wat — wat-side StdInService program.
;;
;; Arc 170 slice 1f-β-i. Mirrors the Rust `StdInServiceEvent` enum
;; shipped in `src/thread_io.rs` (slice 1f-0b, commit d32a29f):
;;
;;   pub enum StdInServiceEvent {
;;       Read,
;;       Add { thread_id, data_rx, reply_tx },
;;       Remove { thread_id },
;;   }
;;
;; Model:
;;   - Service owns an IOReader (stdin fd 0, or in-memory for tests).
;;   - A control channel (EventRx) accepts Add / Remove events from
;;     the runtime orchestrator (slice 1f-gamma).
;;   - Each registered thread has a data-rx (EventRx) it sends
;;     Event::Read on to request the next parsed form.
;;   - The driver selects over all per-thread data-rxs + control-rx
;;     each iteration, routes by index, dispatches per event variant.
;;
;; Architecture decision (honest delta -- HashMap ordering):
;;   The BRIEF specifies HashMap<ThreadId, (data-rx, reply-tx)> for
;;   the routing table.  HashMap/values iteration order is
;;   non-deterministic in the substrate (backed by
;;   std::collections::HashMap).  select-by-index requires a stable
;;   order so the index maps correctly back to the routing entry.
;;   Therefore the driver carries a
;;   Vector<(ThreadId, EventRx, Sender<HolonAST>)> (RoutingEntry)
;;   instead.  The Routing typealias below documents the conceptual
;;   HashMap type per the BRIEF; the driver state is RoutingVec.
;;
;; Loading order: must load AFTER wat/kernel/channel.wat (uses
;; Sender / Receiver typealiases).

;; ─── ThreadId ─────────────────────────────────────────────────────────────
;;
;; Mirrors `pub type ThreadId = i64` from src/thread_io.rs (slice 1f-0b).
;; Placed here (not in a separate kernel/types.wat) because stdin.wat is
;; the first consumer today; future refactor can lift it out.

(:wat::core::typealias :wat::kernel::ThreadId
  :wat::core::i64)

;; ─── Event enum ───────────────────────────────────────────────────────────
;;
;; Mirrors StdInServiceEvent from src/thread_io.rs verbatim.
;;   Read   -- thread requests next EDN line from fd 0.
;;   Add    -- runtime registers a new thread into routing table.
;;   Remove -- runtime removes a thread from routing table.
;;
;; reply-tx carries the RAW EDN LINE back to the requesting thread.
;; Arc 170 slice 1f-iota — pre-1f-iota the service pre-parsed via
;; (:wat::edn::read) and sent an Arc<HolonAST>; post-1f-iota the
;; substrate-side `(:wat::kernel::readln -> :T)` parses + coerces to
;; the caller's declared T (see src/edn_shim.rs::edn_to_typed_value).
;; The wat-side StdInService now ships the raw line as String.

(:wat::core::enum :wat::kernel::services::StdInService::Event
  (Read)
  (Add
    (thread-id :wat::kernel::ThreadId)
    (data-rx :wat::kernel::Receiver<wat::kernel::services::StdInService::Event>)
    (reply-tx :wat::kernel::Sender<wat::core::String>))
  (Remove
    (thread-id :wat::kernel::ThreadId)))

;; ─── Channel typealiases ───────────────────────────────────────────────────
;;
;; Pattern A typealias-family naming convention (Tx/Rx pair per
;; channel, then a (Tx,Rx) tuple typealias for the channel itself).
;; No whitespace inside <> or :() per WAT-CHEATSHEET.md s.2.

(:wat::core::typealias :wat::kernel::services::StdInService::EventTx
  :wat::kernel::Sender<wat::kernel::services::StdInService::Event>)

(:wat::core::typealias :wat::kernel::services::StdInService::EventRx
  :wat::kernel::Receiver<wat::kernel::services::StdInService::Event>)

;; Conceptual routing type per BRIEF.  Driver uses RoutingVec instead
;; because HashMap/values order is non-deterministic.
;; Inner tuple arg inside <> has no leading ':' per WAT-CHEATSHEET s.1.
;; Reply-tx now carries the raw String line (arc 170 slice 1f-iota).
(:wat::core::typealias :wat::kernel::services::StdInService::Routing
  :wat::core::HashMap<wat::kernel::ThreadId,(wat::kernel::services::StdInService::EventRx,wat::kernel::Sender<wat::core::String>)>)

;; One entry in the ordered routing vector: (thread-id, data-rx, reply-tx).
(:wat::core::typealias :wat::kernel::services::StdInService::RoutingEntry
  :(wat::kernel::ThreadId,wat::kernel::services::StdInService::EventRx,wat::kernel::Sender<wat::core::String>))

;; Ordered vector of routing entries.  Index i in this vec maps to
;; index i in the select set built from the data-rxs.
(:wat::core::typealias :wat::kernel::services::StdInService::RoutingVec
  :wat::core::Vector<wat::kernel::services::StdInService::RoutingEntry>)

;; What spawn returns: (Thread<nil,nil>, ControlTx).
;; Caller holds ControlTx to send Add / Remove events; drops it when
;; done => control-rx disconnects => driver exits.
(:wat::core::typealias :wat::kernel::services::StdInService::Spawn
  :(wat::kernel::Thread<wat::core::nil,wat::core::nil>,wat::kernel::services::StdInService::EventTx))


;; ─── Helper: extract data-rxs from routing vec ────────────────────────────
;;
;; Builds a Vector<EventRx> parallel to routing-vec (index i in
;; the result corresponds to entry i in routing-vec).  The result is
;; fed to (:wat::kernel::select ...) alongside [control-rx].
(:wat::core::define
  (:wat::kernel::services::StdInService/routing-rxs
    (routing-vec :wat::kernel::services::StdInService::RoutingVec)
    -> :wat::core::Vector<wat::kernel::services::StdInService::EventRx>)
  (:wat::core::map routing-vec
    (:wat::core::fn
      [entry <- :wat::kernel::services::StdInService::RoutingEntry]
       -> :wat::kernel::services::StdInService::EventRx
      (:wat::core::second entry))))


;; ─── Helper: handle Event::Add ────────────────────────────────────────────
;;
;; Appends a new RoutingEntry to routing-vec.  Returns the new vec.
(:wat::core::define
  (:wat::kernel::services::StdInService/handle-add
    (routing-vec :wat::kernel::services::StdInService::RoutingVec)
    (thread-id :wat::kernel::ThreadId)
    (data-rx :wat::kernel::services::StdInService::EventRx)
    (reply-tx :wat::kernel::Sender<wat::core::String>)
    -> :wat::kernel::services::StdInService::RoutingVec)
  (:wat::core::conj routing-vec
    (:wat::core::Tuple thread-id data-rx reply-tx)))


;; ─── Helper: handle Event::Remove ─────────────────────────────────────────
;;
;; Filters out the entry whose thread-id matches.  Returns the new vec.
(:wat::core::define
  (:wat::kernel::services::StdInService/handle-remove
    (routing-vec :wat::kernel::services::StdInService::RoutingVec)
    (target-id :wat::kernel::ThreadId)
    -> :wat::kernel::services::StdInService::RoutingVec)
  (:wat::core::filter routing-vec
    (:wat::core::fn
      [entry <- :wat::kernel::services::StdInService::RoutingEntry]
       -> :wat::core::bool
      (:wat::core::not
        (:wat::core::= (:wat::core::first entry) target-id)))))


;; ─── Helper: handle Event::Read at index idx ──────────────────────────────
;;
;; Called when select fires at idx < len(routing-vec):
;;   1. Read next line from reader (blocking on fd 0).
;;   2. On Some(line): send the RAW line via reply-tx.  The substrate-
;;      side (:wat::kernel::readln -> :T) parses the line as EDN and
;;      coerces to T (arc 170 slice 1f-iota; see
;;      src/edn_shim.rs::edn_to_typed_value).  Pre-1f-iota this helper
;;      ran (:wat::edn::read line) and sent the parsed HolonAST.
;;   3. On None (EOF): reply-tx disconnects when service shuts down;
;;      no special handling in this slice (slice 1f-gamma handles cascade).
;; Returns unit.
(:wat::core::define
  (:wat::kernel::services::StdInService/handle-read
    (routing-vec :wat::kernel::services::StdInService::RoutingVec)
    (reader :wat::io::IOReader)
    (idx :wat::core::i64)
    -> :wat::core::nil)
  (:wat::core::match (:wat::core::get routing-vec idx) -> :wat::core::nil
    ((:wat::core::Some entry)
      (:wat::core::let
        [reply-tx
          (:wat::core::third entry)
         line-opt
          (:wat::io::IOReader/read-line reader)]
        (:wat::core::match line-opt -> :wat::core::nil
          ((:wat::core::Some line)
            (:wat::core::Result/expect -> :wat::core::nil
              (:wat::kernel::send reply-tx line)
              "StdInService/handle-read: reply-tx disconnected -- thread died?"))
          (:wat::core::None
            ;; EOF on fd 0: callers recv returns disconnected when service
            ;; scope drops.  No special action needed in this slice.
            ()))))
    (:wat::core::None
      ;; idx out of range -- degenerate; cannot happen in practice.
      ())))


;; ─── Dispatch helper ──────────────────────────────────────────────────────
;;
;; Handles the four cases after select fires:
;;   (Ok (Some event)) at data idx  => handle-read + recurse
;;   (Ok (Some event)) at ctrl idx  => handle Add / Remove + recurse
;;   (Ok None) at data idx          => prune entry + recurse
;;   (Ok None) at ctrl idx          => control-rx gone => exit
;;   (Err _) at any idx             => prune / exit same as None
;;
;; Extracted from the loop body per one-let*-per-function rule.
(:wat::core::define
  (:wat::kernel::services::StdInService/dispatch
    (routing-vec :wat::kernel::services::StdInService::RoutingVec)
    (reader :wat::io::IOReader)
    (control-rx :wat::kernel::services::StdInService::EventRx)
    (idx :wat::core::i64)
    (maybe :wat::kernel::CommResult<wat::kernel::services::StdInService::Event>)
    -> :wat::core::nil)
  (:wat::core::let
    [routing-len
      (:wat::core::length routing-vec)
     is-ctrl
      (:wat::core::= idx routing-len)]
    (:wat::core::match maybe -> :wat::core::nil
      ;; ── Fire with a value ──────────────────────────────────────────
      ((:wat::core::Ok (:wat::core::Some event))
        (:wat::core::if is-ctrl -> :wat::core::nil
          ;; Control channel fired -- Add or Remove.
          (:wat::core::match event -> :wat::core::nil
            ((:wat::kernel::services::StdInService::Event::Add t-id d-rx r-tx)
              (:wat::kernel::services::StdInService/loop
                (:wat::kernel::services::StdInService/handle-add
                  routing-vec t-id d-rx r-tx)
                reader
                control-rx))
            ((:wat::kernel::services::StdInService::Event::Remove t-id)
              (:wat::kernel::services::StdInService/loop
                (:wat::kernel::services::StdInService/handle-remove
                  routing-vec t-id)
                reader
                control-rx))
            ((:wat::kernel::services::StdInService::Event::Read)
              ;; Read on control channel is unexpected; ignore + recurse.
              (:wat::kernel::services::StdInService/loop
                routing-vec reader control-rx)))
          ;; Data channel fired -- Read expected.
          (:wat::core::let
            [_ (:wat::kernel::services::StdInService/handle-read
                  routing-vec reader idx)]
            (:wat::kernel::services::StdInService/loop
              routing-vec reader control-rx))))
      ;; ── Clean disconnect ────────────────────────────────────────────
      ((:wat::core::Ok :wat::core::None)
        (:wat::core::if is-ctrl -> :wat::core::nil
          ;; Control channel disconnected => shutdown.
          ()
          ;; Data channel disconnected => prune entry + recurse.
          (:wat::kernel::services::StdInService/loop
            (:wat::std::list::remove-at routing-vec idx)
            reader
            control-rx)))
      ;; ── Peer panic / cascade ────────────────────────────────────────
      ((:wat::core::Err _died)
        (:wat::core::if is-ctrl -> :wat::core::nil
          ;; Control channel panicked => shutdown.
          ()
          ;; Data channel panicked => prune entry + recurse.
          (:wat::kernel::services::StdInService/loop
            (:wat::std::list::remove-at routing-vec idx)
            reader
            control-rx))))))


;; ─── Driver loop ──────────────────────────────────────────────────────────
;;
;; Each iteration:
;;   1. Build select set: data-rxs from routing-vec ++ [control-rx].
;;      (routing-vec may be empty; then select set = [control-rx].)
;;   2. select blocks until one receiver fires.
;;   3. Forward to dispatch helper by index.
;;   4. Recurse with updated routing-vec.
;;
;; Shutdown: control-rx disconnects (ControlTx dropped by caller) =>
;; dispatch exits the recursion => return unit => Thread<nil,nil>
;; delivers unit on its output Sender.
;;
;; One let* per function per feedback_simple_forms_per_func.
(:wat::core::define
  (:wat::kernel::services::StdInService/loop
    (routing-vec :wat::kernel::services::StdInService::RoutingVec)
    (reader :wat::io::IOReader)
    (control-rx :wat::kernel::services::StdInService::EventRx)
    -> :wat::core::nil)
  (:wat::core::let
    [data-rxs
      (:wat::kernel::services::StdInService/routing-rxs routing-vec)
     select-set
      (:wat::core::concat data-rxs
        (:wat::core::Vector
          :wat::kernel::services::StdInService::EventRx
          control-rx))
     chosen
      (:wat::kernel::select select-set)
     idx
      (:wat::core::first chosen)
     maybe
      (:wat::core::second chosen)]
    (:wat::kernel::services::StdInService/dispatch
      routing-vec reader control-rx idx maybe)))


;; ─── spawn ────────────────────────────────────────────────────────────────
;;
;; Creates the StdInService program.  Returns (Thread<nil,nil>, ControlTx)
;; per SERVICE-PROGRAMS.md lockstep.
;;
;;   reader -- the IOReader for fd 0 (or in-memory for tests).
;;
;; Caller:
;;   (let [spawn (StdInService/spawn reader)
;;         thr   (first spawn)
;;         ctrl  (second spawn)]
;;     ;; Send Add / Remove events on ctrl.
;;     ;; Drop ctrl => service shuts down.
;;     (Thread/join-result thr))
(:wat::core::define
  (:wat::kernel::services::StdInService/spawn
    (reader :wat::io::IOReader)
    -> :wat::kernel::services::StdInService::Spawn)
  (:wat::core::let
    [ctrl-pair
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdInService::Event 1)
     ctrl-tx
      (:wat::core::first ctrl-pair)
     ctrl-rx
      (:wat::core::second ctrl-pair)
     thr
      (:wat::kernel::spawn-thread
        (:wat::core::fn
          [_in <- :rust::crossbeam_channel::Receiver<wat::core::nil>
           _out <- :rust::crossbeam_channel::Sender<wat::core::nil>]
           -> :wat::core::nil
          (:wat::kernel::services::StdInService/loop
            (:wat::core::Vector
              :wat::kernel::services::StdInService::RoutingEntry)
            reader
            ctrl-rx)))]
    (:wat::core::Tuple thr ctrl-tx)))
