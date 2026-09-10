;; RELAND 9 STOP-2: expand sift-rules-defsvc separately. Not the template family.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src  (:wat::io::read-file "tests/services/probe_arc278_sift_rules.wat")
     tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome.Forms {:forms f} f]
             [:wat::core::ReadOutcome.Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     ch   (:wat::core::ast->children tree)
     svcs (:wat::core::into []
             (:wat::core::filter
               (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
                 (:wat::core::= (:wat::fix::head-name n) ":wat::query::sift-rules-defsvc"))
               ch))
     svc  (:wat::core::first svcs)
     expanded (:wat::core::macroexpand-1 svc)]
    (:wat::io::write-file "/tmp/r9/sift-expand.wat" (:wat::core::ast->source expanded))
    (:wat::kernel::println "WROTE /tmp/r9/sift-expand.wat")
    nil))
