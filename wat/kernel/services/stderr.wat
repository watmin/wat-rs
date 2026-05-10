;; wat/kernel/services/stderr.wat — wat-side StdErrService program.
;;
;; Arc 170 slice 1f-β-iii. Mirrors the Rust `StdErrServiceEvent` enum
;; shipped in `src/thread_io.rs` (slice 1f-0b, commit d32a29f):
;;
;;   pub enum StdErrServiceEvent {
;;       Write { line: String },
;;       Add { thread_id, data_rx, ack_tx },
;;       Remove { thread_id },
;;   }
;;
;; Model:
;;   - Service owns an IOWriter (stderr fd 2, or in-memory for tests).
;;   - A control channel (EventRx) accepts Add / Remove events from
;;     the runtime orchestrator (slice 1f-gamma).
;;   - Each registered thread has a data-rx (EventRx) it sends
;;     Event::Write on to request that a line be written to fd 2;
;;     the service acks via ack-tx (Sender<nil>) after writing.
;;   - The driver selects over all per-thread data-rxs + control-rx
;;     each iteration, routes by index, dispatches per event variant.
;;
;; Per TIERS.md doctrine: fd 2 carries only panic-cascade EDN inside
;; wat-land. wat-cli has zero direct stderr writes.
;;
;; Architecture decision (honest delta -- HashMap ordering):
;;   The BRIEF specifies HashMap<ThreadId, (data-rx, ack-tx)> for
;;   the routing table.  HashMap/values iteration order is
;;   non-deterministic in the substrate (backed by
;;   std::collections::HashMap).  select-by-index requires a stable
;;   order so the index maps correctly back to the routing entry.
;;   Therefore the driver carries a
;;   Vector<(ThreadId, EventRx, Sender<nil>)> (RoutingEntry)
;;   instead.  The Routing typealias below documents the conceptual
;;   HashMap type per the BRIEF; the driver state is RoutingVec.
;;
;; Data-channel dispatch shape (explicit per slice 1f-β-ii lesson):
;;   The data-channel receives the full Event enum.  Dispatch on
;;   data-rx fire must match event and handle three arms:
;;   - Write { line } → the productive case
;;   - Add → no-op (Add should arrive only on control-rx; defensive arm)
;;   - Remove → no-op (same reason; defensive arm)
;;
;; Loading order: must load AFTER wat/kernel/channel.wat (uses
;; Sender / Receiver typealiases) and AFTER wat/kernel/services/stdout.wat
;; (which loads after stdin.wat which defines :wat::kernel::ThreadId).

;; ─── Event enum ───────────────────────────────────────────────────────────
;;
;; Mirrors StdErrServiceEvent from src/thread_io.rs verbatim.
;;   Write  -- thread requests that `line` be written to fd 2.
;;   Add    -- runtime registers a new thread into routing table.
;;   Remove -- runtime removes a thread from routing table.
;;
;; ack-tx carries unit (nil) back to the requesting thread confirming
;; the write completed.

(:wat::core::enum :wat::kernel::services::StdErrService::Event
  (Write (line :wat::core::String))
  (Add
    (thread-id :wat::kernel::ThreadId)
    (data-rx :wat::kernel::Receiver<wat::kernel::services::StdErrService::Event>)
    (ack-tx :wat::kernel::Sender<wat::core::nil>))
  (Remove
    (thread-id :wat::kernel::ThreadId)))

;; ─── Channel typealiases ───────────────────────────────────────────────────
;;
;; Follows the wat/console.wat:38 typealias-family naming convention.
;; No whitespace inside <> or :() per WAT-CHEATSHEET.md s.2.

(:wat::core::typealias :wat::kernel::services::StdErrService::EventTx
  :wat::kernel::Sender<wat::kernel::services::StdErrService::Event>)

(:wat::core::typealias :wat::kernel::services::StdErrService::EventRx
  :wat::kernel::Receiver<wat::kernel::services::StdErrService::Event>)

;; Conceptual routing type per BRIEF.  Driver uses RoutingVec instead
;; because HashMap/values order is non-deterministic.
;; Inner tuple arg inside <> has no leading ':' per WAT-CHEATSHEET s.1.
(:wat::core::typealias :wat::kernel::services::StdErrService::Routing
  :wat::core::HashMap<wat::kernel::ThreadId,(wat::kernel::services::StdErrService::EventRx,wat::kernel::Sender<wat::core::nil>)>)

;; One entry in the ordered routing vector: (thread-id, data-rx, ack-tx).
(:wat::core::typealias :wat::kernel::services::StdErrService::RoutingEntry
  :(wat::kernel::ThreadId,wat::kernel::services::StdErrService::EventRx,wat::kernel::Sender<wat::core::nil>))

;; Ordered vector of routing entries.  Index i in this vec maps to
;; index i in the select set built from the data-rxs.
(:wat::core::typealias :wat::kernel::services::StdErrService::RoutingVec
  :wat::core::Vector<wat::kernel::services::StdErrService::RoutingEntry>)

;; What spawn returns: (Thread<nil,nil>, ControlTx).
;; Caller holds ControlTx to send Add / Remove events; drops it when
;; done => control-rx disconnects => driver exits.
(:wat::core::typealias :wat::kernel::services::StdErrService::Spawn
  :(wat::kernel::Thread<wat::core::nil,wat::core::nil>,wat::kernel::services::StdErrService::EventTx))


;; ─── Helper: extract data-rxs from routing vec ────────────────────────────
;;
;; Builds a Vector<EventRx> parallel to routing-vec (index i in
;; the result corresponds to entry i in routing-vec).  The result is
;; fed to (:wat::kernel::select ...) alongside [control-rx].
(:wat::core::define
  (:wat::kernel::services::StdErrService/routing-rxs
    (routing-vec :wat::kernel::services::StdErrService::RoutingVec)
    -> :wat::core::Vector<wat::kernel::services::StdErrService::EventRx>)
  (:wat::core::map routing-vec
    (:wat::core::fn
      [entry <- :wat::kernel::services::StdErrService::RoutingEntry]
       -> :wat::kernel::services::StdErrService::EventRx
      (:wat::core::second entry))))


;; ─── Helper: handle Event::Add ────────────────────────────────────────────
;;
;; Appends a new RoutingEntry to routing-vec.  Returns the new vec.
(:wat::core::define
  (:wat::kernel::services::StdErrService/handle-add
    (routing-vec :wat::kernel::services::StdErrService::RoutingVec)
    (thread-id :wat::kernel::ThreadId)
    (data-rx :wat::kernel::services::StdErrService::EventRx)
    (ack-tx :wat::kernel::Sender<wat::core::nil>)
    -> :wat::kernel::services::StdErrService::RoutingVec)
  (:wat::core::conj routing-vec
    (:wat::core::Tuple thread-id data-rx ack-tx)))


;; ─── Helper: handle Event::Remove ─────────────────────────────────────────
;;
;; Filters out the entry whose thread-id matches.  Returns the new vec.
(:wat::core::define
  (:wat::kernel::services::StdErrService/handle-remove
    (routing-vec :wat::kernel::services::StdErrService::RoutingVec)
    (target-id :wat::kernel::ThreadId)
    -> :wat::kernel::services::StdErrService::RoutingVec)
  (:wat::core::filter routing-vec
    (:wat::core::fn
      [entry <- :wat::kernel::services::StdErrService::RoutingEntry]
       -> :wat::core::bool
      (:wat::core::not
        (:wat::core::= (:wat::core::first entry) target-id)))))


;; ─── Helper: handle Event::Write at index idx ─────────────────────────────
;;
;; Called when select fires at idx < len(routing-vec):
;;   1. Write the line to the writer (appends newline via writeln).
;;   2. Send unit via the matched ack-tx to confirm completion.
;; Returns unit.
;;
;; Honest delta -- ack-tx zero-payload send:
;;   `(:wat::kernel::send ack-tx ())` is the correct shape for
;;   Sender<wat::core::nil>; unit literal `()` is the nil value.
(:wat::core::define
  (:wat::kernel::services::StdErrService/handle-write
    (routing-vec :wat::kernel::services::StdErrService::RoutingVec)
    (writer :wat::io::IOWriter)
    (idx :wat::core::i64)
    (line :wat::core::String)
    -> :wat::core::nil)
  (:wat::core::match (:wat::core::get routing-vec idx) -> :wat::core::nil
    ((:wat::core::Some entry)
      (:wat::core::let
        [ack-tx
          (:wat::core::third entry)
         _bytes
          (:wat::io::IOWriter/writeln writer line)]
        (:wat::core::Result/expect -> :wat::core::nil
          (:wat::kernel::send ack-tx ())
          "StdErrService/handle-write: ack-tx disconnected -- thread died?")))
    (:wat::core::None
      ;; idx out of range -- degenerate; cannot happen in practice.
      ())))


;; ─── Dispatch helper ──────────────────────────────────────────────────────
;;
;; Handles the four cases after select fires:
;;   (Ok (Some event)) at data idx  => handle-write + recurse
;;   (Ok (Some event)) at ctrl idx  => handle Add / Remove + recurse
;;   (Ok None) at data idx          => prune entry + recurse
;;   (Ok None) at ctrl idx          => control-rx gone => exit
;;   (Err _) at any idx             => prune / exit same as None
;;
;; Extracted from the loop body per one-let*-per-function rule.
(:wat::core::define
  (:wat::kernel::services::StdErrService/dispatch
    (routing-vec :wat::kernel::services::StdErrService::RoutingVec)
    (writer :wat::io::IOWriter)
    (control-rx :wat::kernel::services::StdErrService::EventRx)
    (idx :wat::core::i64)
    (maybe :wat::kernel::CommResult<wat::kernel::services::StdErrService::Event>)
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
            ((:wat::kernel::services::StdErrService::Event::Add t-id d-rx a-tx)
              (:wat::kernel::services::StdErrService/loop
                (:wat::kernel::services::StdErrService/handle-add
                  routing-vec t-id d-rx a-tx)
                writer
                control-rx))
            ((:wat::kernel::services::StdErrService::Event::Remove t-id)
              (:wat::kernel::services::StdErrService/loop
                (:wat::kernel::services::StdErrService/handle-remove
                  routing-vec t-id)
                writer
                control-rx))
            ((:wat::kernel::services::StdErrService::Event::Write _line)
              ;; Write on control channel is unexpected; ignore + recurse.
              (:wat::kernel::services::StdErrService/loop
                routing-vec writer control-rx)))
          ;; Data channel fired -- Write expected.
          (:wat::core::match event -> :wat::core::nil
            ((:wat::kernel::services::StdErrService::Event::Write line)
              (:wat::core::let
                [_ (:wat::kernel::services::StdErrService/handle-write
                      routing-vec writer idx line)]
                (:wat::kernel::services::StdErrService/loop
                  routing-vec writer control-rx)))
            ((:wat::kernel::services::StdErrService::Event::Add _t _d _a)
              ;; Add on data channel is unexpected; ignore + recurse.
              (:wat::kernel::services::StdErrService/loop
                routing-vec writer control-rx))
            ((:wat::kernel::services::StdErrService::Event::Remove _t)
              ;; Remove on data channel is unexpected; ignore + recurse.
              (:wat::kernel::services::StdErrService/loop
                routing-vec writer control-rx)))))
      ;; ── Clean disconnect ────────────────────────────────────────────
      ((:wat::core::Ok :wat::core::None)
        (:wat::core::if is-ctrl -> :wat::core::nil
          ;; Control channel disconnected => shutdown.
          ()
          ;; Data channel disconnected => prune entry + recurse.
          (:wat::kernel::services::StdErrService/loop
            (:wat::std::list::remove-at routing-vec idx)
            writer
            control-rx)))
      ;; ── Peer panic / cascade ────────────────────────────────────────
      ((:wat::core::Err _died)
        (:wat::core::if is-ctrl -> :wat::core::nil
          ;; Control channel panicked => shutdown.
          ()
          ;; Data channel panicked => prune entry + recurse.
          (:wat::kernel::services::StdErrService/loop
            (:wat::std::list::remove-at routing-vec idx)
            writer
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
  (:wat::kernel::services::StdErrService/loop
    (routing-vec :wat::kernel::services::StdErrService::RoutingVec)
    (writer :wat::io::IOWriter)
    (control-rx :wat::kernel::services::StdErrService::EventRx)
    -> :wat::core::nil)
  (:wat::core::let
    [data-rxs
      (:wat::kernel::services::StdErrService/routing-rxs routing-vec)
     select-set
      (:wat::core::concat data-rxs
        (:wat::core::Vector
          :wat::kernel::services::StdErrService::EventRx
          control-rx))
     chosen
      (:wat::kernel::select select-set)
     idx
      (:wat::core::first chosen)
     maybe
      (:wat::core::second chosen)]
    (:wat::kernel::services::StdErrService/dispatch
      routing-vec writer control-rx idx maybe)))


;; ─── spawn ────────────────────────────────────────────────────────────────
;;
;; Creates the StdErrService program.  Returns (Thread<nil,nil>, ControlTx)
;; per SERVICE-PROGRAMS.md lockstep.
;;
;;   writer -- the IOWriter for fd 2 (or in-memory for tests).
;;
;; Caller:
;;   (let [spawn (StdErrService/spawn writer)
;;         thr   (first spawn)
;;         ctrl  (second spawn)]
;;     ;; Send Add / Remove events on ctrl.
;;     ;; Drop ctrl => service shuts down.
;;     (Thread/join-result thr))
(:wat::core::define
  (:wat::kernel::services::StdErrService/spawn
    (writer :wat::io::IOWriter)
    -> :wat::kernel::services::StdErrService::Spawn)
  (:wat::core::let
    [ctrl-pair
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdErrService::Event 1)
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
          (:wat::kernel::services::StdErrService/loop
            (:wat::core::Vector
              :wat::kernel::services::StdErrService::RoutingEntry)
            writer
            ctrl-rx)))]
    (:wat::core::Tuple thr ctrl-tx)))
