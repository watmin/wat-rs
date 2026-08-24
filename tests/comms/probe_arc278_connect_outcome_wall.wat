;; tests/comms/probe_arc278_connect_outcome_wall.wat — co-located fixture for
;; probe_arc278_connect_outcome_wall.rs, slurped via startup_beside(file!()).
;;
;; Arc 278 peer-lifecycle Strike 4 — the connect' OUTCOME WALL (the LAST peer wall).
;; `connect'` used to return a bare `(Peer' :- [S R])` and RAISE on its handleable failures;
;; it now returns a matchable `(:wat::kernel::ConnectOutcome :- [S R])` (::Connected[(Peer' :- [S R])]
;; · ::Refused[Failure] · ::Rejected[Failure] · ::Failed[Failure]). These fns RETURN the
;; raw ConnectOutcome so the Rust probe can assert on it STRUCTURALLY (Value::Enum field
;; extraction).
;;
;; Thread tier is single-threaded-drivable: `listener'` mints a bounded(1) crossbeam
;; rendezvous, thread-tier `connect'` ships its one-way connect-request into the empty
;; slot WITHOUT blocking and returns the client Peer' — Connected, no second thread, no
;; deadlock. Dropping the listener (the rendezvous Receiver) makes the send disconnected
;; → Refused.

;; HAPPY DIAL → ConnectOutcome::Connected[peer]. `connect'` queues a connect-request in
;; the live rendezvous slot (the listener in `pair` is still alive) and wraps the client
;; Peer'.
(:wat::core::defn :user::connect-happy [] -> (:wat::kernel::ConnectOutcome :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     addr (:wat::spawn::Bound/address pair)]
    (:wat::kernel::connect addr)))

;; Extract ONLY the address (the rendezvous Sender); the enclosing `Bound` (holding the
;; listener's crossbeam Receiver) is dropped when this helper returns — so the rendezvous
;; has no live receiver left.
(:wat::core::defn :user::orphaned-address [] -> (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])
  (:wat::spawn::Bound/address
    (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)))

;; RETRYABLE TRANSPORT → ConnectOutcome::Refused[cause]. connect' on an address whose
;; listener (the only rendezvous Receiver) was dropped → crossbeam send Disconnected →
;; Refused (no listener / rendezvous gone), NOT a raise the dialer unwinds past.
(:wat::core::defn :user::connect-refused [] -> (:wat::kernel::ConnectOutcome :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [addr (:user::orphaned-address)]
    (:wat::kernel::connect addr)))
