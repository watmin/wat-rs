;; wat-scripts/scratch-pad/255-home4-string-carve-metadata.wat — arc 255 home #4
;; phase 2 (the string carve): dump `(metadata-of <fqdn>)` for all 19
;; `:wat::string::*` verbs carved into `src/intrinsic/string.rs`, one per line,
;; so row 3's acceptance ("metadata-of answers for each of the 19") can be
;; read directly off stdout. Scratch, per holon/CLAUDE.md's `.wat` scratch
;; convention (not the ephemeral session tmp). Direct calls (not routed
;; through a user fn) — `metadata-of` takes the FQDN keyword literal
;; in-position, same as `probe_arc255_ivb1_structured_doc.wat`.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── :wat::string::contains? ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::contains?))
    (:wat::kernel::println "── :wat::string::starts-with? ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::starts-with?))
    (:wat::kernel::println "── :wat::string::ends-with? ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::ends-with?))
    (:wat::kernel::println "── :wat::string::length ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::length))
    (:wat::kernel::println "── :wat::string::trim ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::trim))
    (:wat::kernel::println "── :wat::string::to-lowercase ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::to-lowercase))
    (:wat::kernel::println "── :wat::string::to-uppercase ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::to-uppercase))
    (:wat::kernel::println "── :wat::string::pascal->kebab ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::pascal->kebab))
    (:wat::kernel::println "── :wat::string::pascal->kebab-in ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::pascal->kebab-in))
    (:wat::kernel::println "── :wat::string::kebab->pascal-in ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::kebab->pascal-in))
    (:wat::kernel::println "── :wat::string::subs ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::subs))
    (:wat::kernel::println "── :wat::string::split ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::split))
    (:wat::kernel::println "── :wat::string::join ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::join))
    (:wat::kernel::println "── :wat::string::concat ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::concat))
    (:wat::kernel::println "── :wat::string::interpolate ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::interpolate))
    (:wat::kernel::println "── :wat::string::declare-acronyms ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::declare-acronyms))
    (:wat::kernel::println "── :wat::string::to-i64 ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::to-i64))
    (:wat::kernel::println "── :wat::string::to-f64 ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::to-f64))
    (:wat::kernel::println "── :wat::string::to-bool ──")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::string::to-bool))))
