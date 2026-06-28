;; tests/macros/probe_deftest_hermetic_isolation.wat — co-located fixture for
;; probe_deftest_hermetic_isolation.rs, slurped via startup_beside(file!()).
;;
;; All four probe programs merged under one :user::main. All prelude content
;; is inside (:wat::core::forms ...) so the parent world is not contaminated.

;; Probe 1: parent symbol table untouched by prelude struct.
(:wat::test::deftest-hermetic :test::g::my-hermetic-test
  ((:wat::core::defstruct :test::g::IsolatedType [field <- :wat::core::i64]))
  (:wat::core::do
    (:test::g::IsolatedType/new 42)
    :wat::core::nil))

;; Probe 2: cross-test prelude isolation — two tests each declare :test::g::SharedName.
(:wat::test::deftest-hermetic :test::g::first-hermetic-test
  ((:wat::core::defstruct :test::g::SharedName [value <- :wat::core::i64]))
  :wat::core::nil)

(:wat::test::deftest-hermetic :test::g::second-hermetic-test
  ((:wat::core::defstruct :test::g::SharedName [label <- :wat::core::String]))
  :wat::core::nil)

;; Probe 3: test fn visible in parent; prelude content invisible.
(:wat::test::deftest-hermetic :test::g::visible-test
  ((:wat::core::defstruct :test::g::HiddenStruct
     [x <- :wat::core::i64
      y <- :wat::core::i64])
   (:wat::core::defn :test::g::hidden-helper [] -> :test::g::HiddenStruct
     (:test::g::HiddenStruct/new 0 0)))
  :wat::core::nil)

;; Probe 4: make-deftest-hermetic with define prelude freezes cleanly; parent isolated.
(:wat::test::make-deftest-hermetic :deftest-g-isolated
  (
   (:wat::core::defn :test::g::run-inner [] -> :wat::kernel::RunResult
     (:wat::test::run-hermetic
                    (:wat::kernel::println "hello")))
  ))

(:deftest-g-isolated :test::g::using-make-deftest-hermetic
  :wat::core::nil)

