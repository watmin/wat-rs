(:wat::core::defn :user::c01a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::core::i64"))))
(:wat::core::defn :user::c01b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::core::String"))))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::core::Vector<wat::core::i64>"))))
(:wat::core::defn :user::c03a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":i64"))))
(:wat::core::defn :user::c03b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":String"))))
(:wat::core::defn :user::c03c [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":bool"))))
(:wat::core::defn :user::c04 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::kernel::services::StdErrService::Req"))))
(:wat::core::defn :user::c05a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::kernel::services::StdErrService::Req"))))
(:wat::core::defn :user::c05b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::kernel::services::StdInService::Req"))))
(:wat::core::defn :user::c06 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":wat::holon::HolonAST"))))
(:wat::core::defn :user::c07a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":T"))))
(:wat::core::defn :user::c07b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":K"))))
;; c08, c09: raise at RUNTIME for these inputs — startup succeeds. STONE-the-last-mint —
;; c08a/c08b's angle-bracket strings now refuse at `keyword-node` itself (the minting wall),
;; one door earlier than `keyword/to-type-form`'s own parser used to catch them; c09's
;; trailing-`::` path carries no angle bracket, so it is unaffected and still reaches
;; `keyword/to-type-form` to raise there. Both assertions (`.rs`) only check `.is_err()`,
;; so neither needed to move.
(:wat::core::defn :user::c08a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":Stream<wat::core::i64>"))))
(:wat::core::defn :user::c08b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":T<wat::core::i64>"))))
(:wat::core::defn :user::c09 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node ":foo::"))))
