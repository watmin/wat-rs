;; tests/comms/probe_arc278_accept_outcome_wall.wat — co-located fixture for
;; probe_arc278_accept_outcome_wall.rs, slurped via startup_beside(file!()).
;;
;; Arc 278 peer-lifecycle Strike 3 — the accept' OUTCOME WALL. `accept'` used to
;; return a bare `Peer'<R,S>` and RAISE on its handleable failures; it now returns a
;; matchable `:wat::kernel::AcceptOutcome<R,S>` (::Accepted[Peer'<R,S>] · ::Closed ·
;; ::Failed[Failure]). These fns RETURN the raw AcceptOutcome so the Rust probe can
;; assert on it STRUCTURALLY (Value::Enum field extraction).
;;
;; Thread tier is single-threaded-drivable: `listener'` mints a bounded(1) crossbeam
;; rendezvous, thread-tier `connect'` ships its one-way connect-request into the empty
;; slot WITHOUT blocking and returns the client Peer', then `accept'` dequeues it —
;; Accepted, no second thread, no deadlock.

;; HAPPY PATH → AcceptOutcome::Accepted[peer]. `connect'` queues a connect-request in
;; the rendezvous slot; `accept'` dequeues + wraps the authorized server Peer'.
(:wat::core::defn :user::accept-happy [] -> (:wat::kernel::AcceptOutcome :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [pair    (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l       (:wat::spawn::Bound/listener pair)
     addr    (:wat::spawn::Bound/address pair)
     _client (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))]
    (:wat::kernel::accept l)))

;; Extract ONLY the listener (rx); the enclosing `Bound` (holding the address's
;; crossbeam Sender) is dropped when this helper returns — so the rendezvous has no
;; live senders left.
(:wat::core::defn :user::orphaned-listener [] -> (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])
  (:wat::spawn::Bound/listener
    (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)))

;; CLEAN TERMINAL → AcceptOutcome::Closed[]. accept' on a listener whose address
;; (the only rendezvous Sender) was dropped → crossbeam recv Disconnected → Closed,
;; NOT a raise the server loop unwinds past.
(:wat::core::defn :user::accept-closed [] -> (:wat::kernel::AcceptOutcome :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [l (:user::orphaned-listener)]
    (:wat::kernel::accept l)))
