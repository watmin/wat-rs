;; Does peer-pid work on the BRACKET's actual worker peer (spawn-runner -> Process'),
;; as opposed to a connect'-derived unified Peer'? Decides whether the shadowdancer's
;; peer-pid is correct for the real use case.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* x 2))
     ;; the PROCESS worker peer — exactly what map-worker holds
     pp   (:wat::spawn::Locus/spawn-runner (:wat::spawn::process) work)
     _    (:wat::kernel::println "process spawn-runner peer-pid:")
     _    (:wat::kernel::println (:wat::kernel::peer-pid pp))
     ;; the THREAD worker peer
     tp   (:wat::spawn::Locus/spawn-runner (:wat::spawn::thread) work)
     _    (:wat::kernel::println "thread spawn-runner peer-pid:")
     _    (:wat::kernel::println (:wat::kernel::peer-pid tp))]
    nil))
