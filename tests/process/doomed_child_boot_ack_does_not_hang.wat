;; Co-located fixture for doomed_child_boot_ack_does_not_hang.rs.
;; The spawn post-spawn callback is never reached — the child dies at execve.
;; The fn exists so spawn_process_peer has a typed handle to pass.

(:wat::core::defn :my::noop-post-spawn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)
