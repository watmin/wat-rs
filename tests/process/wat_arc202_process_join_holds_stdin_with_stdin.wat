;; tests/process/wat_arc202_process_join_holds_stdin_with_stdin.wat
;; NEGATIVE fixture — startup must FAIL with DefRestrictedCallerNotAllowed only
;; (ProcessJoinHoldsStdinSender must NOT fire — Process/stdin is present).
;; User-namespace fn calls both Process/stdin AND Process/join-result in the same let scope.

(:wat::core::defn :my::arc202::negative-stdin-present [proc <- :wat::kernel::Process<wat::core::nil,wat::core::nil>] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>
  (:wat::core::let
    [stdin-w (:wat::kernel::Process/stdin proc)
     joined  (:wat::kernel::Process/join-result proc)]
    joined))

