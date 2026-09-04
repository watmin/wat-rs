;; Arc 255 Stone "the three special-form tables" — baseline probe for the 9 bucket-2 names
;; (registered, no @syntax) BEFORE any edit. Rider capture for before/after comparison.
(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::Option/expect))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::Option/try))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::Result/expect))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::Result/try))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::and))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::if))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::core::or))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::form::matches?))
    (:wat::kernel::println (:wat::runtime::signature-of-defn :wat::holon::literal))))
