;; tests/process/lifeline_orphan_clean_via_substrate.wat — co-located fixture for
;; lifeline_orphan_clean_via_substrate.rs. startup_beside(file!()) world — built BEFORE any
;; fork; the supervisor inherits it via fork's copy-on-write semantics (unchanged from the
;; original hand-built-AST version).
;;
;; :my::launch spawns the grandchild: creates an unbounded channel, keeps the sender alive,
;; blocks on recv. The lifeline mechanism (not SIGTERM) wakes the blocked recv when the
;; supervisor exits — see the Rust driver's module doc for the full design.

;; Arc 212 stone δ-comm-positions law: a bare recv as a let RHS whose name is
;; never consumed is CommCallOutOfPosition. `Result/expect` is the correct
;; (not merely legal-satisfying) wrapping here: per the module doc, the same
;; RecvOutcome::Shutdown outcome wakes this recv (via the lifeline mechanism
;; rather than SIGTERM) and must propagate as a RuntimeError — exactly
;; Result/expect's panic-on-Err behavior.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
                [[tx rx] (:wat::kernel::make-channel :wat::core::nil)
                 _       (:wat::core::Result/expect (:wat::kernel::recv rx) "recv woke via shutdown cascade")]
                :wat::core::nil)))))
