;; tests/wat_lang/probe_arc241_stone12_defalias.wat
;; Stone 241.12 — :wat::core::defalias mint: all three positive cases (C01–C03).

(:wat::core::defn :app::greet [] -> wat.type/String "hello")
(:wat::core::defalias :app::salutation :app::greet)
(:wat::core::defn :test::call-alias    [] -> wat.type/String (:app::salutation))
(:wat::core::defn :test::call-original [] -> wat.type/String (:app::greet))
(:wat::core::defalias :user::my-length :wat::core::length)
(:wat::core::defn :test::use-my-length [] -> wat.type/i64 (:user::my-length [1 2 3]))
