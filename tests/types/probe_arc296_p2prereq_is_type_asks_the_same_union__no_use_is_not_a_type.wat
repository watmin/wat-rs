;; ⛔ THE OVER-REACH DETECTOR — the SAME name as the subject, with NO use!.
;; use! is a PER-PROGRAM declaration. A program that did not declare it does not
;; have it, exactly as resolve/walk.rs's coverage rule holds for call heads.
;; Seeding from the build-time registry instead of from use! would turn this true.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :rust::sqlite::Connection)))
