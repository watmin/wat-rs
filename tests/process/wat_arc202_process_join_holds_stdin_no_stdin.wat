;; tests/process/wat_arc202_process_join_holds_stdin_no_stdin.wat
;; NEGATIVE fixture — startup must FAIL with ProcessJoinHoldsStdinSender + DefRestrictedCallerNotAllowed.
;; User-namespace fn calls Process/join-result without any Process/stdin extraction in scope.

(:wat::core::defn :my::arc202::negative-no-stdin [proc <- :wat::kernel::Process<wat::core::nil,wat::core::nil>] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::LociDiedError>>
  (:wat::core::let
    [joined (:wat::kernel::Process/join-result proc)]
    joined))

