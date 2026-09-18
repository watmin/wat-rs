;; mode-parity CONTROL: both `--check` and run accept.
;; Copied from wat_cli__check_good.wat (zero-arg main, not a bare nil body —
;; UselessMain wall). --check never runs the println.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "check-good"))
