;; RELAND 9 STOP-1: expand the POSITIVE toy-journal defservice (map-form impls)
;; and look at generated Reply / Success / Op constructions.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [src  (:wat::io::read-file "tests/services/probe_arc278_journal_surface.wat")
     tree (:wat::core::match (:wat::core::read-string src)
             [:wat::core::ReadOutcome::Forms {:forms f} f]
             [:wat::core::ReadOutcome::Malformed {:cause c}
               (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     ch   (:wat::core::ast->children tree)
     svcs (:wat::core::into []
             (:wat::core::filter
               (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
                 (:wat::core::= (:wat::fix::head-name n) ":wat::service::defservice"))
               ch))
     svc  (:wat::core::first svcs)
     expanded (:wat::core::macroexpand-1 svc)]
    (:wat::io::write-file "/tmp/r9/journal-expand.wat" (:wat::core::ast->source expanded))
    (:wat::kernel::println "WROTE /tmp/r9/journal-expand.wat")
    nil))
