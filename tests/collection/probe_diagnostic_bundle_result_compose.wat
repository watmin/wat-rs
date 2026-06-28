;; tests/collection/probe_diagnostic_bundle_result_compose.wat — co-located fixture.
;; Disconfirms "Bundle's Result return blocks canonical Bind(Atom, Bundle) defrecord composition."

;; Probe 1: Bind composes with Bundle via Result/expect → bool (is? returns true)
(:wat::core::defn :t::probe1-bind-composes [] -> :wat::core::bool
  (:wat::core::let
    [field-a (:wat::holon::Bind
               (:wat::holon::Atom (:wat::holon::to-holon "a"))
               (:wat::holon::Atom (:wat::holon::to-holon 1)))
     field-b (:wat::holon::Bind
               (:wat::holon::Atom (:wat::holon::to-holon "b"))
               (:wat::holon::Atom (:wat::holon::to-holon 2)))
     inner-bundle (:wat::core::Result/expect
                    (:wat::holon::Bundle [field-a field-b])
                    "Bundle should not overflow")
     instance (:wat::holon::Bind
                (:wat::holon::Atom (:wat::holon::to-holon "test::Foo"))
                inner-bundle)]
    (:wat::holon::is? instance "test::Foo")))

;; Probe 2: canonical instance shape preserves inner Bundle → i64 (statement-length = 3)
(:wat::core::defn :t::probe2-inner-bundle-preserved [] -> :wat::core::i64
  (:wat::core::let
    [field-a (:wat::holon::Bind
               (:wat::holon::Atom (:wat::holon::to-holon "a"))
               (:wat::holon::Atom (:wat::holon::to-holon 1)))
     field-b (:wat::holon::Bind
               (:wat::holon::Atom (:wat::holon::to-holon "b"))
               (:wat::holon::Atom (:wat::holon::to-holon 2)))
     field-c (:wat::holon::Bind
               (:wat::holon::Atom (:wat::holon::to-holon "c"))
               (:wat::holon::Atom (:wat::holon::to-holon 3)))
     inner-bundle (:wat::core::Result/expect
                    (:wat::holon::Bundle [field-a field-b field-c])
                    "Bundle should not overflow")]
    (:wat::holon::statement-length inner-bundle)))
