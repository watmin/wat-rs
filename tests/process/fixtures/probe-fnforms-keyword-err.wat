;; probe-fnforms-keyword-err.wat — negative control: `fn-forms` given a keyword naming no
;; registered fn (`:no::such::fn`) must refuse, never reach the println below.
;;
;; DISPOSITION (255.75) — negative, by design (the `-err` suffix and the deliberately-bogus
;; keyword name the intent): a plain `.wat` (AMEND: this file starts up clean — `startup_from_file` succeeds, the
;; failure is runtime-only — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` instead, so the "every probe runs, exit
;; 0" gate does not require it), driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting the TypeMismatch names
;; `:wat::kernel::fn-forms` and `:no::such::fn`, and that "should not reach here" never prints.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [forms (:wat::kernel::fn-forms (:wat::keyword::from-string "no::such::fn") :x)]
    (:wat::kernel::println "should not reach here")))
