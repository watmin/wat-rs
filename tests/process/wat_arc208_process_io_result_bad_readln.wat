;; tests/process/wat_arc208_process_io_result_bad_readln.wat
;; NEGATIVE fixture — startup must FAIL with CommCallOutOfPosition (or Process/readln) for T7.
;; Process/readln appearing directly as a do-body expression is the forbidden pattern.

(:wat::core::defn :user::bad-readln
  [peer <- :wat::kernel::ProcessPeer<wat::core::String,wat::core::String>]
  -> :wat::core::String
  (:wat::core::do
    (:wat::kernel::Process/readln peer)
    "fallback"))
