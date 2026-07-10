;; Real-body trivial :user::main for the harness happy-path tests (arc-170 wall: no bare nil).
;; Lives in a .wat fixture so no_inlined_wat (which scans .rs string literals) never sees it.
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::let [_argv (:wat::runtime::argv)] nil))
