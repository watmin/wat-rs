;; tests/process/wat_arc208_process_io_result_bad_println.wat
;; NEGATIVE fixture — startup must FAIL with CommCallOutOfPosition (or Process/println) for T6.
;; Process/println appearing directly as a do-body expression is the forbidden pattern.

(:wat::core::defn :user::bad-println
  [peer <- :wat::kernel::ProcessPeer<wat::core::String,wat::core::String>]
  -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::Process/println peer "hello")
    :wat::core::nil))
