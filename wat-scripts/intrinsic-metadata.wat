;; wat-scripts/intrinsic-metadata.wat — dump (metadata-of <fqdn>) as EDN.
;;
;; Arc 255.1b-iv-c GROUND-IT-FIRST: dogfood the reflection surface to SEE the
;; real metadata map before deciding the keyword→enum representation for the
;; closed-domain values :kind / :defined-in / :layer (runtime.rs:10120-10122).
;;
;; Usage:
;;   ./target/release/wat ./wat-scripts/intrinsic-metadata.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match
    (:wat::runtime::metadata-of :wat::intrinsic::examples) -> :wat::core::nil
    (:wat::core::None
      (:wat::kernel::eprintln
        "intrinsic-metadata: no metadata for :wat::intrinsic::examples"))
    ((:wat::core::Some meta)
      (:wat::kernel::println meta))))
