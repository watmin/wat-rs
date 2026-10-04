;; Excursus 003 strike D3, GD2a — UserMainMissing's real producer.
;;
;; No `:user::main` declared at all. `startup_from_source`'s `:user::main` wall is
;; conditional on main being declared (`startup_bare()` passes cleanly with none),
;; so this freezes successfully; `invoke_user_main` then raises `UserMainMissing`
;; because `USER_MAIN_PATH` is absent from the frozen `SymbolTable`.

(:wat::core::defn :my::noop [] -> :wat::core::i64 1)
