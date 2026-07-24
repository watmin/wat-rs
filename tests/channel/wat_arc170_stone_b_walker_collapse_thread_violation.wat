;; NEGATIVE fixture — a user-namespace fn reaching for the restricted :wat::kernel::Thread/join-result.
;; This program MUST FAIL to freeze: arc-198's walk_for_restricted_call refuses a non-:wat:: caller
;; (DefRestrictedCallerNotAllowed). The probe asserts the rejection names the verb + the variant.
(:wat::core::defn :my::test::call-thread-join [thr <- :wat::kernel::Thread<wat::core::nil,wat::core::nil>] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::LociDiedError>> (:wat::kernel::Thread/join-result thr))

