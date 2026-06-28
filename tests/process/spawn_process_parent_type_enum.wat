;; tests/process/spawn_process_parent_type_enum.wat
;; fixture for spawn_process_parent_type.rs probe_spawn_process_inherits_parent_enum
;; startup_from_file — parent-declared enum visible in child via edn::read (Arc 170 Gap F-3).

(:wat::core::defenum :test::proto::Color
  :Red
  :Green
  :Blue)

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defenum :test::proto::Color
        :Red
        :Green
        :Blue)
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [s "#test.proto.Color/Red nil"
           _ (:wat::edn::read s)]
          nil)))))

