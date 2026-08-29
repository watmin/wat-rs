;; Scratch probe — arc 255 Stone P6-c-W2, acceptance row 4.
;;
;; Direct, statically-checked calls to the W2 candidates at their legal arity.
;; Output must be byte-identical before and after homing — only the DISPATCH
;; mechanism changes, never the returned value. `program::env` prints only the
;; two fields stable across a run on one machine (`cpu-count`, `peer-kind`) —
;; `started-at`/`process-id`/`os-thread-id` are host/run-specific by design and
;; would never be byte-identical across two invocations regardless of homing.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "stream-empty->vec= "
      (:wat::edn::write (:wat::core::stream->vec [] (:wat::stream::empty)))))
    (:wat::kernel::println (:wat::string::concat "stream-cons->vec= "
      (:wat::edn::write (:wat::core::stream->vec []
        (:wat::stream::cons 1 (:wat::stream::cons 2 (:wat::stream::empty)))))))
    (:wat::core::match (:wat::stream::next (:wat::stream::cons 42 (:wat::stream::empty)))
      ((:wat::stream::NextOutcome::Item v rest) (:wat::kernel::println (:wat::string::concat "stream-next-item= " (:wat::edn::write v)
        " rest->vec= " (:wat::edn::write (:wat::core::stream->vec [] rest)))))
      (:wat::stream::NextOutcome::Exhausted (:wat::kernel::println "stream-next-item= EXHAUSTED (unexpected)")))
    (:wat::core::match (:wat::stream::next (:wat::stream::empty))
      ((:wat::stream::NextOutcome::Item v rest) (:wat::kernel::println (:wat::string::concat "stream-next-empty= UNEXPECTED item " (:wat::edn::write v))))
      (:wat::stream::NextOutcome::Exhausted (:wat::kernel::println "stream-next-empty= Exhausted")))
    (:wat::core::do
      (:wat::core::let [e (:wat::program::env)]
        (:wat::kernel::println (:wat::string::concat "program-env-cpu-count= " (:wat::edn::write (:wat::program::Env/cpu-count e))
          " peer-kind= " (:wat::edn::write (:wat::program::Env/peer-kind e))))))))
