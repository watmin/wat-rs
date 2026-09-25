;; Stone 255.34 — ConnectOutcome says what happened.
;; A dropped thread listener is Closed: the rendezvous does not come back.
;; A dropped process listener is Closed: the autobind name is gone (ECONNREFUSED).
;; Pre-stone both rows printed Refused. The inert-wire row lives in
;; probe_arc255_29_thread_address_to_process.wat (Undialable). The wrong-peer
;; row is the live connect in src/kernel/address.rs.
(:wat::core::defn :user::show
  [label <- :wat::core::String
   o <- (:wat::kernel::ConnectOutcome :- [:wat::core::i64 :wat::core::i64])]
  -> :wat::core::nil
  (:wat::core::match o
    [:wat::kernel::ConnectOutcome.Connected {:peer _p}
      (:wat::kernel::println (:wat::string::concat label " Connected"))]
    [:wat::kernel::ConnectOutcome.Closed {:cause c}
      (:wat::kernel::println (:wat::string::concat label " Closed " (:wat::kernel::Failure/message c)))]
    [:wat::kernel::ConnectOutcome.Undialable {:cause c}
      (:wat::kernel::println (:wat::string::concat label " Undialable " (:wat::kernel::Failure/message c)))]
    [:wat::kernel::ConnectOutcome.WrongPeer {:cause c}
      (:wat::kernel::println (:wat::string::concat label " WrongPeer " (:wat::kernel::Failure/message c)))]
    [:wat::kernel::ConnectOutcome.Failed {:cause c}
      (:wat::kernel::println (:wat::string::concat label " Failed " (:wat::kernel::Failure/message c)))]))

(:wat::core::defn :user::orphaned-thread [] -> (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])
  (:wat::spawn::Bound/address
    (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)))

(:wat::core::defn :user::orphaned-process [] -> (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Wire])
  (:wat::spawn::Bound/address
    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::show "thread-dropped" (:wat::kernel::connect (:user::orphaned-thread)))
    (:user::show "process-dead" (:wat::kernel::connect (:user::orphaned-process)))))
