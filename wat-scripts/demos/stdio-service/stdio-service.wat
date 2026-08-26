;; stdio-service.wat — a stateful service whose wire IS stdin/stdout.
;;
;; THE SHAPE: `:user::main` tail-calls into a frame processor. One frame per
;; iteration; the state rides as a parameter; TCO turns the recursion into a jump,
;; so the loop is flat no matter how long the conversation runs.
;;
;; That is the whole trick for consuming an UNBOUNDED stream through a BOUNDED
;; reader. Each frame is one MTU on the wire (512 KiB — `DEFAULT_MAX_FRAME_BYTES`);
;; the loop is how you cross an arbitrary number of them without growing a stack.
;;
;;   :user::main
;;      └─ TCO →  :repl::serve state
;;                  read ONE frame  →  dispatch  →  reply  →  TCO → :repl::serve state'
;;
;; ── HOW THIS RELATES TO defservice ────────────────────────────────────────────
;;
;; This is the loop `defservice` generates for you, written out. `defservice` owns
;; the channel, derives dispatch from a surface, and threads state through the same
;; kind of tail call. If something DIALS you — another service, a bracket worker —
;; use it; hand-rolled IPC is exactly what it exists to replace.
;;
;; This file is for the other case: a program at the end of a PIPE. There is no
;; peer to dial and no surface to satisfy, because after handover fd 0/1 are simply
;; yours. `(:wat::program::self-peer)` is unavailable here and says so plainly —
;; *"only valid inside a spawned process service; root has no owner-link"* — so a
;; root program that wants a conversation writes this loop.
;;
;; ── RUN IT ────────────────────────────────────────────────────────────────────
;;   ./target/release/wat wat-scripts/demos/stdio-service/stdio-service.wat \
;;     < wat-scripts/demos/stdio-service/session.edn
;;
;;   …or interactively, one frame per line:
;;   ./target/release/wat wat-scripts/demos/stdio-service/stdio-service.wat
;;   #repl.Cmd/Bump [5]
;;   #repl.Cmd/Show []
;;   #repl.Cmd/Quit []

;; ── The protocol ──────────────────────────────────────────────────────────────
;;
;; Requests and replies are enums, so the reader matches EVERY variant — a `_` arm
;; on an enum is illegal here (109's NOTE-full-enum-match-mandatory-no-wildcard-arm),
;; which means adding a command later breaks the BUILD rather than falling through
;; at runtime to a peer who sent something you forgot to handle.
(:wat::core::defenum :repl::Cmd :wat::enum::Pure
  :Bump [by <- :wat::core::i64]
  :Show []
  :Quit [])

(:wat::core::defenum :repl::Reply :wat::enum::Pure
  :Value [n <- :wat::core::i64]
  :Bye   [final <- :wat::core::i64])

;; ── The serve loop ────────────────────────────────────────────────────────────
;;
;; `readln` takes exactly ONE frame and decodes it to `:repl::Cmd` — the match arms
;; are what tell the checker the frame's type, so a frame that is not a Cmd fails as
;; a LOCATED decode error rather than as a value that quietly means something else.
;;
;; Every non-terminal arm ends in a tail call carrying the next state. The terminal
;; arm returns `nil`, which ends the conversation and the process with it.
(:wat::core::defn :repl::serve
  [count <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)

    ;; A frame arrived — decode it to `:repl::Cmd` and dispatch. This inner match
    ;; is the whole non-terminal body: every arm ends in a tail call carrying the
    ;; next state.
    ((:wat::kernel::ReadlnOutcome::Datum __datum)
      (:wat::core::match __datum

        ;; Mutate: fold the delta into the state and carry it forward.
        ((:repl::Cmd::Bump by)
          (:wat::core::let [next (:wat::i64::+ count by)]
            (:wat::kernel::println (:repl::Reply::Value next))
            (:repl::serve next)))

        ;; Read: reply with the current state, carry it unchanged.
        ((:repl::Cmd::Show)
          (:wat::core::do
            (:wat::kernel::println (:repl::Reply::Value count))
            (:repl::serve count)))

        ;; Terminate: say goodbye and RETURN. No tail call — the loop ends here, and
        ;; the caller (`:user::main`) returns nil, so the process exits 0.
        ((:repl::Cmd::Quit)
          (:wat::kernel::println (:repl::Reply::Bye count)))))

    ;; The client closed the conversation — the same terminal shape as `Quit`,
    ;; just without a goodbye to send. Returning nil ends the process cleanly.
    (:wat::kernel::ReadlnOutcome::Eof     nil)

    ;; A process-wide stop was requested — the same clean end, named distinctly
    ;; so a reader can tell "the client hung up" from "we were told to stop".
    (:wat::kernel::ReadlnOutcome::Stopped nil)))

;; ── main is one tail call ─────────────────────────────────────────────────────
;;
;; Everything `main` does is hand control to the loop with the initial state. The
;; program's entire behaviour is the frame processor.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:repl::serve 0))
