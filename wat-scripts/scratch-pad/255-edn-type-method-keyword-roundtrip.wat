;; Witness: Type/method keywords write with ONE slash and read back.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [written (:wat::edn::write :wat::holon::Hologram/make)]
    (:wat::core::do
      (:wat::kernel::println written)
      (:wat::kernel::println
        (:wat::edn::write (:wat::edn::read written))))))
