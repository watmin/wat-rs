;; Malformed config setter (bad type) — set-capacity-mode! takes a keyword;
;; passing a string triggers ConfigError::BadType, a startup failure. See
;; tests/cli/wat_cli.rs::startup_error_bubbles_up_as_exit_3.
(:wat::config::set-capacity-mode! "oops")
