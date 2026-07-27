;; Prints the argv ambient so the cli test can assert its exact structure.
;; See tests/cli/wat_cli.rs::argv_passes_shell_args_through_to_user_main.
;;
;; Arc 170's own purpose: `(:wat::runtime::argv)` carries the WHOLE OS argv to
;; `:user::main` — argv[0] = the resolved wat binary, argv[1] = the entry file,
;; argv[2..] = whatever else the caller said, unedited.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::argv)))
