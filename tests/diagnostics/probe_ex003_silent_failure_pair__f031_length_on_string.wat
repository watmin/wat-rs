;; Probe fixture — the-little-wat F-031: `length` on a String.
;;
;; The checker ACCEPTS this program (`wat --check` -> rc 0); the runtime REJECTS it
;; (`wat` -> rc 1, TypeMismatch inside :wat::core::length). That (0, 1) pair is the
;; silent-failure signature this probe banks. Do not "fix" this file — the file is
;; correct AS A SPECIMEN; the finding is that wat accepts it.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::length "abc")))
