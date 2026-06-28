;; NEGATIVE fixture — a user-namespace fn reaching for the restricted :wat::kernel::Process/join-result.
;; This program MUST FAIL to freeze (DefRestrictedCallerNotAllowed; non-:wat:: caller). The probe asserts
;; the rejection names the verb + the variant.
(:wat::core::defn :my::test::call-process-join [proc <- :wat::kernel::Process<wat::core::nil,wat::core::nil>] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>> (:wat::kernel::Process/join-result proc))

