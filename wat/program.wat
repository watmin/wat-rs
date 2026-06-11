;; wat/program.wat — arc 258 A2: :wat::program::Env as a typed extensible recordtype base.
;;
;; Replaces the Rust-builtin typealias (HashMap<keyword, HolonAST>) with a
;; proper record definition, enabling subtype extension for user programs.
;;
;; Loading order: must load AFTER wat/Record.wat (uses :wat::Record::def)
;; and :wat::time::Instant is a builtin already available at startup.

(:wat::Record::def :wat::program::Env [wat.started-at <- :wat::time::Instant])
