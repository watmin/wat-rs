;; Regression guard (arc 140 origin) — a genuinely-unknown call head at a
;; deftest body is caught at SUB-PROGRAM resolve/freeze time with the standard
;; "not a builtin, not a registered function" diagnostic. The name exists
;; nowhere (not a sandbox-scope leak — arc 140 slice 2's separate
;; SandboxScopeLeak diagnostic, for a name that exists in an outer scope but
;; not the inner one, was retired excursus 003 strike G item 1: measured
;; dead, its only gate — `SymbolTable::outer_symbols` — was never set by any
;; real production path; this is the typo case: nowhere at all).
;;
;; The exact diagnostic substring is the live resolve-pass output, pinned by
;; the arc 211c audit (2026-05-18) — `should-panic "not a builtin"` below.
(:wat::test::should-panic "not a builtin")
(:wat::test::deftest :wat-tests::core::unknown-call-head-panics
  
  (:wat::test::assert-eq (:totally::made::up::name 42) 42))
