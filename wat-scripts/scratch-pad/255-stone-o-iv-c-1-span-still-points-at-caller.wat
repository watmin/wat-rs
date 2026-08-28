;; wat-scripts/scratch-pad/255-stone-o-iv-c-1-span-still-points-at-caller.wat — arc 255
;; Stone O-iv-c-1, acceptance row 2. Same instrument as
;; `255-stone-q-2-the-threaded-span-must-be-used.wat` (Stone Q-2's row 2): an UNCAUGHT
;; `apply` call (not wrapped in `:wat::eval-ast!`) so the process dies and the printed
;; `RuntimeError` carries its `:location` — the caught `EvalError` a wat program sees via
;; `eval-ast!`/`match` never exposes location (only `:kind`/`:message`), so this uncaught
;; path is the only instrument that can show it.
;;
;; `:wat::holon::OnlineSubspace/dim` is migrated this stone and KEEPS its trailing `&Span`
;; (it calls `require_subspace`, which needs a location). Two call sites below, `span-a`
;; and `span-b`, raise the SAME `require_subspace` TypeMismatch (arg is an `i64`, not an
;; `OnlineSubspace`) from two different source lines. If the span were stale/hardcoded,
;; both would report the SAME `:line`/`:col`; if it is the real per-call span Stone Q
;; threads through, they differ.
;;
;; Run with `./target/release/wat <this file> <case>`, case in {span-a, span-b}.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     case (:wat::core::Option/expect (:wat::core::get argv 2) "usage: <this file> <case>")]
    (:wat::core::cond
      ((:wat::core::= case "span-a")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::holon::OnlineSubspace/dim (:wat::core::Vector :wat::core::Value 42)))))

      ((:wat::core::= case "span-b")
       (:wat::kernel::println (:wat::edn::write
         (:wat::core::apply :wat::holon::OnlineSubspace/dim (:wat::core::Vector :wat::core::Value 42)))))

      (:else
       (:wat::kernel::println (:wat::string::concat "unknown case: " case))))))
