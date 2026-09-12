;; AcceptOutcome::Closed — bind a listener, drop the address (no one will dial),
;; then accept. If the substrate reports Closed when the rendezvous is gone,
;; we print it. A hang is also a measurement (wrapper timeout).
;;
;; Run:
;;   timeout 3 ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-accept-closed.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    ;; Extract the listener and drop Bound (and its address) before accept.
    [lis (:wat::spawn::Bound/listener
           (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64))
     o   (:wat::kernel::accept lis)]
    (:wat::core::match o
      ((:wat::kernel::AcceptOutcome::Accepted _)
        (:wat::kernel::println "accept-no-dial=Accepted"))
      (:wat::kernel::AcceptOutcome::Closed
        (:wat::kernel::println "accept-no-dial=Closed"))
      ((:wat::kernel::AcceptOutcome::Failed _)
        (:wat::kernel::println "accept-no-dial=Failed")))))
