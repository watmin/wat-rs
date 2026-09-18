;; excursus 001-sns-sqs `a-mention-in-a-quoted-form-is-not-a-call` — THE WITNESS.
;;
;; One metadata-map, on the user's OWN entry point. Nine stdlib `…::service-forms`
;; bodies QUOTE `:user::main` into a `(:wat::core::forms …)` child-program template,
;; and the arc-198 mention walker fired on each of those quoted leaves — turning a
;; `"hi"`/exit-0 program into 9 `DefRestrictedCallerNotAllowed` errors inside SIX
;; stdlib files the author has never seen and cannot edit.
;;
;; A name inside a quoted template is DATA: resolved in a different program, at a
;; different time, by a different caller. A CALLER-restriction on it checks the
;; wrong thing at the wrong time. This must load clean.
(:wat::core::defn :user::main {:restricted-to [:my::]} [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
