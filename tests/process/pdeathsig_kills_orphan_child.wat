;; tests/process/pdeathsig_kills_orphan_child.wat — co-located fixture for
;; pdeathsig_kills_orphan_child.rs. startup_beside(file!()) world — built BEFORE any fork;
;; the supervisor inherits it via fork's copy-on-write semantics (unchanged from the original
;; hand-built-AST version).
;;
;; :my::launch spawns the grandchild: creates an unbounded channel, keeps the sender alive,
;; blocks on recv. PDEATHSIG-delivered SIGTERM fires the Slice B cascade, waking the recv.

;; Arc 212 stone δ-comm-positions law: a bare recv as a let RHS whose name is
;; never consumed is CommCallOutOfPosition. `Result/expect` is the correct
;; (not merely legal-satisfying) wrapping here: per the module doc, when the
;; PDEATHSIG cascade wakes this recv it returns RecvOutcome::Shutdown, which
;; must propagate as a RuntimeError so spawn_process_child_branch exits with
;; EXIT_RUNTIME_ERROR — exactly what Result/expect's panic-on-Err gives us.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
                [[tx rx] (:wat::kernel::make-channel :wat::core::nil)
                 _       (:wat::core::Result/expect (:wat::kernel::recv rx) "recv woke via shutdown cascade")]
                :wat::core::nil)))))
