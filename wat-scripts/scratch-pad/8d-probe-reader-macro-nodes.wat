;; 8d probe — the shape of a reader-macro node (`~x`), for class B's cure.
;; `~` is 1 source char; the reader synthesizes a form whose head names the
;; 19-char `:wat::core::unquote`. fix-text-apply refuses to splice when the
;; rule takes old-text from that NAME. Recorded per the scratch-pad convention.
(:wat::core::defn :user::show [src <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [tree (:wat::core::match (:wat::core::read-string src)
            [:wat::core::ReadOutcome.Forms {:forms __f} __f]
            [:wat::core::ReadOutcome.Malformed {:cause __c}
              (:wat::kernel::assertion-failed! :message "read failed")])
     top  (:wat::core::first (:wat::core::ast->children tree))
     kind (:wat::core::ast-kind top)]
    (:wat::core::if (:wat::core::= kind "list")
      (:wat::core::let [h (:wat::core::first (:wat::core::ast->children top))]
        (:wat::kernel::println (:wat::string::concat
          "src=" src " | top=list | head-kind=" (:wat::core::ast-kind h)
          " | head-name=" (:wat::core::ast-name h))))
      (:wat::kernel::println (:wat::string::concat "src=" src " | top-kind=" kind)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:user::show "~x")
    (:user::show "(a ~b)")
    (:user::show "x")))
