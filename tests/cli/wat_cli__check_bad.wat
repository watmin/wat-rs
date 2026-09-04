;; A program `wat --check` rejects: produces 2 type-check errors — one
;; TypeMismatch (argument #1 of i64::+ is a String) + one ReturnTypeMismatch
;; (the body produces i64; the signature declares nil). See tests/cli/wat_cli.rs
;; (arc 115 slice 1 `wat --check` mode): check_mode_exits_nonzero_on_bad_program,
;; check_output_edn_emits_record_per_diagnostic,
;; check_output_json_emits_record_per_diagnostic.
;;
;; Arc 278 IPC de-prime — RE-SPECIMENED. The prior specimen was
;; `(:wat::kernel::send no-such-thing 42)`, whose first diagnostic was
;; CommCallOutOfPosition, emitted by the arc-110 `validate_comm_positions`
;; walker. That walker existed solely to police the raw `send`/`recv` verbs and
;; was annihilated with them; with the verb gone the unknown callee defers to a
;; runtime UnknownFunction (`--check` is not a complete RED arbiter) and the
;; body's type becomes a fresh var, so the ReturnTypeMismatch stopped firing
;; too — this fixture silently ceased to be a bad program at all.
;;
;; The replacement is deliberately anchored on two STRUCTURAL, permanent
;; diagnostics — argument type-checking and return type-checking — rather than
;; on any one verb's walker, so it cannot be quietly hollowed out by a future
;; retirement the way its predecessor was. The SHAPE the tests measure is
;; preserved exactly: two records, one carrying a `:callee` field and one a
;; `:function` field.
;;
;; Arc 278 C20 — THE ORDER OF THOSE TWO RECORDS FLIPPED, and it is now a stated property
;; rather than an accident. Check errors leave `check_program` sorted into SOURCE order, so
;; the ReturnTypeMismatch (span = the WHOLE body form) precedes the TypeMismatch at the
;; argument nested inside it.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::i64::+ "not-a-number" 1))
