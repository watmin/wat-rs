;; wat-scripts/fixes/variant-vector-to-tagged-map.wat — arc 296 H-2.
;; Self-hosted codemod: rewrite wat SOURCE that embeds the retired variant
;; wire `#ns.Enum/Variant […]` as a tagged literal form.
;;
;; Live wat programs construct variants via `(:Enum::Variant …)`, not EDN
;; tags. The census of `#wat.core.Option/` in `.wat` is comments and
;; EDN-in-strings (read-foreign fixtures). A form-tree walk cannot rewrite
;; string contents; those sites are named in the SCORE.
;;
;; Idempotent. Dry-run on a /tmp copy, then:
;;   printf '["pathA" …]\n' | cargo wat ./wat-scripts/fixes/variant-vector-to-tagged-map.wat
;;
;; This file is the recorded migration. It currently identity-rewrites: there
;; is no wat tagged-literal form for the old wire in the corpus (R21: do not
;; invent a rewrite of a shape that is not a wat form).

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  src)

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[variant-map] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome::Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome::Eof {}
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)]
      [:wat::kernel::ReadlnOutcome::Stopped {}
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)])))
