;; Stone 255.35 — a dropped thread listener is Closed.
;; The address (the only rendezvous sender) is dropped with the Bound.
;; Pre-stone this row was already Closed. A stop is a different fact
;; and is driven from the sibling .rs, because wat has no cascade verb.
(:wat::core::defn :user::orphaned-listener [] -> (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])
  (:wat::spawn::Bound/listener
    (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::accept (:user::orphaned-listener))
    [:wat::kernel::AcceptOutcome.Accepted {:peer _p}
      (:wat::kernel::println "thread-dropped Accepted")]
    [:wat::kernel::AcceptOutcome.Closed {}
      (:wat::kernel::println "thread-dropped Closed")]
    [:wat::kernel::AcceptOutcome.Stopped {}
      (:wat::kernel::println "thread-dropped Stopped")]
    [:wat::kernel::AcceptOutcome.Failed {:cause c}
      (:wat::kernel::println (:wat::string::concat "thread-dropped Failed " (:wat::kernel::Failure/message c)))]))
