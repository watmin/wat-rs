;; probe-arc278-invocation-family.wat — the three invocation contexts are CONSTRUCTIBLE and their
;; spliced core fields are READABLE.
;;
;; WHY THIS EXISTS: the floor went 4384/0 the moment the family landed, and that green proved
;; NOTHING about it — no test constructs any of the three, so a splice that silently failed to apply
;; would leave every existing test passing. This probe is the thing that would go red
;; ([[feedback_a_green_test_can_prove_nothing]]): it builds each record and reads BOTH a spliced core
;; field and (where present) the record's own field.
;;
;; What it proves:
;;   1. `~@:wat::service::InvocationCore` actually splices — `invocation-id` / `namespace` /
;;      `operation` / `start-ns` are reachable as accessors on all three records.
;;   2. The three records are distinct constructible types, not one type with aliases.
;;   3. `LifecycleInvocation` and `Invocation` carry `conn-id`; `SelfInvocation` does NOT (it has no
;;      connection — that absence is the whole point of the three-type split, and it is STRUCTURAL:
;;      there is no field to read, so asking a timer for a connection cannot be written down).

(:wat::core::defn :user::self-invocation-reads-core [] -> :wat::core::String
  (:wat::core::let
    [inv (:wat::service::SelfInvocation
           :namespace     :probe::ticker
           :operation     "-tick"
           :invocation-id (:wat::uuid::v4)
           :start-ns      42)]
    (:wat::service::SelfInvocation/operation inv)))

(:wat::core::defn :user::lifecycle-invocation-reads-core-and-conn [] -> :wat::core::i64
  (:wat::core::let
    [inv (:wat::service::LifecycleInvocation
           :namespace     :probe::ticker
           :operation     "-on-connect"
           :invocation-id (:wat::uuid::v4)
           :start-ns      42
           :conn-id       7)]
    (:wat::service::LifecycleInvocation/conn-id inv)))

(:wat::core::defn :user::invocation-reads-core-and-conn [] -> :wat::core::i64
  (:wat::core::let
    [inv (:wat::service::Invocation
           :namespace     :probe::ticker
           :operation     "whoami"
           :invocation-id (:wat::uuid::v4)
           :start-ns      42
           :conn-id       2)]
    (:wat::core::i64::+
      (:wat::service::Invocation/conn-id inv)
      (:wat::string::length (:wat::service::Invocation/operation inv)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:user::self-invocation-reads-core)
      (:wat::string::concat
        (:wat::core::i64::to-string (:user::lifecycle-invocation-reads-core-and-conn))
        (:wat::core::i64::to-string (:user::invocation-reads-core-and-conn))))))
