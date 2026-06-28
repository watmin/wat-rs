;; tests/process/spawn_process_parent_type_parametric.wat
;; fixture for spawn_process_parent_type.rs probe_spawn_process_inherits_parametric_type
;; startup_from_file — parent-declared parametric struct visible in child via edn::read (Arc 170 Gap F-3).

(:wat::core::defstruct :test::proto::Wrapper<E>
  [label <- :wat::core::String
   value <- :wat::core::i64])

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defstruct :test::proto::Wrapper<E>
        [label <- :wat::core::String
         value <- :wat::core::i64])
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [s "#test.proto/Wrapper {:label :empty :value 42}"
           _ (:wat::edn::read s)]
          nil)))))

