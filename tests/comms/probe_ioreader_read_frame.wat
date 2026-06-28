;; tests/comms/probe_ioreader_read_frame.wat — co-located fixture slurped via startup_beside(file!()).
;; Gate 3: type-checker accepts IOReader/read-frame and startup succeeds.
;; Creates a reader from a string literal and calls read-frame — must type-check (startup Ok).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [r (:wat::io::IOReader/from-string "42\n")]
    (:wat::io::IOReader/read-frame r)
    nil))
