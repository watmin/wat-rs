;; RecvOutcome::Closed — a clean child that prints nothing closes the wire.
;; RecvOutcome::Lost   — a child that raise! dies; the parent sees Lost.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-closed-lost.wat

(:wat::core::defn :cs::label-recv :- [O]
  [o <- (:wat::kernel::RecvOutcome :- [:O])] -> :wat::core::String
  (:wat::core::match o
    ((:wat::kernel::RecvOutcome::Message _) "Message")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Lost _) "Lost")
    ((:wat::kernel::RecvOutcome::Malformed _) "Malformed")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p-closed (:wat::test::spawn-peer (:wat::spawn::process)
                (:wat::core::forms
                  (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     closed (:cs::label-recv (:wat::kernel::recv p-closed))
     p-lost (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::kernel::raise! (:wat::core::Fault/of "crash-surface-lost")))))
     lost (:cs::label-recv (:wat::kernel::recv p-lost))]
    (:wat::kernel::println
      (:wat::core::format "recv-empty-child={c};recv-raise-child={l}"
        :c closed :l lost))))
