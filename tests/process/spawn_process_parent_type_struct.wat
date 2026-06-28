;; tests/process/spawn_process_parent_type_struct.wat
;; fixture for spawn_process_parent_type.rs probe_spawn_process_inherits_parent_struct
;; startup_from_file — parent-declared struct visible in child via edn::read (Arc 170 Gap F-3).

(:wat::core::defstruct :test::proto::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defstruct :test::proto::Point
        [x <- :wat::core::i64
         y <- :wat::core::i64])
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [s "#test.proto/Point {:x 3 :y 4}"
           _ (:wat::edn::read s)]
          nil)))))

