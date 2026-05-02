;; Arc 140 follow-up reproduction — totally-unknown name at deftest body.
;;
;; STATUS (2026-05-03): SHOULD-PANIC pending arc 140 slice 1 (runtime
;; enrichment) + a deeper resolve-pass investigation into why the
;; sub-program freeze doesn't catch this at startup. Today the failure
;; is a generic runtime UnknownFunction; the goal: catch at SUB-PROGRAM
;; resolve / freeze time with the standard "unresolved reference"
;; diagnostic — no special teaching needed since the name is genuinely
;; nowhere (not a sandbox-scope leak; just a typo).
;;
;; Arc 140 slice 2 (the SandboxScopeLeak check rule) handles a
;; DIFFERENT case: name exists in outer scope but not inner. This
;; probe is the OTHER case: name exists nowhere. Both should fail at
;; freeze, but only the first has a teaching diagnostic.

(:wat::test::should-panic "unknown function")
(:wat::test::deftest :wat-tests::tmp::totally-bogus
  ()
  (:wat::test::assert-eq (:totally::made::up::name 42) 42))
