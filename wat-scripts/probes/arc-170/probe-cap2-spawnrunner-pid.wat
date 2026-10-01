;; Does peer-pid work on the BRACKET's actual worker peer (spawn-runner -> Process'),
;; as opposed to a connect'-derived unified Peer'? Decides whether the shadowdancer's
;; peer-pid is correct for the real use case.
;; CLAIM (shape, not digits — a pid is never pinned): a PROCESS spawn-runner peer's
;; peer-pid is Option.Some of a positive i64 (a real OS pid); a THREAD spawn-runner
;; peer's peer-pid is Option.None (no OS pid for a thread peer).
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [work (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::* x 2))
     ;; the PROCESS worker peer — exactly what map-worker holds
     pp   (:wat::spawn::Locus/spawn-runner (:wat::spawn::process) work)
     pp-pid (:wat::kernel::peer-pid pp)
     _    (:wat::core::match pp-pid
            [:wat::core::Option.Some {:value v} (:wat::test::assert-true (:wat::i64::> v 0))]
            [:wat::core::Option.None {}
              (:wat::kernel::assertion-failed! :message "process spawn-runner peer-pid: expected Some, got None")])
     _    (:wat::kernel::println "process spawn-runner peer-pid:")
     _    (:wat::kernel::println pp-pid)
     ;; the THREAD worker peer
     tp   (:wat::spawn::Locus/spawn-runner (:wat::spawn::thread) work)
     tp-pid (:wat::kernel::peer-pid tp)
     _    (:wat::core::match tp-pid
            [:wat::core::Option.None {} nil]
            [:wat::core::Option.Some {:value v}
              (:wat::kernel::assertion-failed! :message "thread spawn-runner peer-pid: expected None, got Some")])
     _    (:wat::kernel::println "thread spawn-runner peer-pid:")
     _    (:wat::kernel::println tp-pid)]
    nil))
