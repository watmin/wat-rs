;; Scratch: 2a2 door smoke. Not a recorded migration.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src "(:wat::core::defenum :usr::E :wat::enum::Pure :Variant [a <- :wat::core::i64 b <- :wat::core::i64] :Unit [])"
     tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms f} f]
             [:wat::core::ReadOutcome.Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     forms (:wat::core::ast->children tree)
     m (:wat::fix::enum-fields "probe" "scratch" forms
         (:wat::core::Vector :- [:wat::core::String] ":usr::E" ":wat::core::Option"))]
    (:wat::core::do
      (:wat::kernel::println
        (:wat::core::str (:wat::fix::enum-fields-get m ":usr::E::Variant")))
      (:wat::kernel::println
        (:wat::core::str (:wat::fix::enum-fields-get m ":usr::E::Unit")))
      (:wat::kernel::println
        (:wat::core::str (:wat::fix::enum-fields-get m ":wat::core::Option::Some")))
      (:wat::kernel::println
        (:wat::core::str (:wat::fix::EnumFields/answered m))))))
