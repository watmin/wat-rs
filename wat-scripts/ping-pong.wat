;; wat-scripts/ping-pong.wat — proof: wat program spawns wat program,
;; bidirectional ping-pong over kernel pipes, both shut down cleanly.
;;
;; Usage:
;;   ./target/release/wat ./wat-scripts/ping-pong.wat
;;
;; Expected output:
;;   round 1: ping → pong
;;   round 2: ping → pong
;;   round 3: ping → pong
;;   round 4: ping → pong
;;   round 5: ping → pong
;;   done — 5 round trips
;;
;; The shape:
;;
;;   wat-cli (Rust binary)
;;     └─ ping-pong.wat (frozen world A) ← we are here
;;          ├─ stdin/stdout/stderr → real OS handles (wat-cli's)
;;          └─ :wat::kernel::spawn-program ./pong.wat
;;               └─ pong.wat (frozen world B, on a thread)
;;                    └─ stdin/stdout/stderr → 3 OS pipe ends
;;
;; Two frozen worlds, three pipes, EDN+newline framing. The parent
;; sends `#demo/Ping {:n N}`; the child responds with
;; `#demo/Pong {:n N}`; the parent verifies the echo matched and
;; logs the round. After N rounds the parent closes the child's
;; stdin via :wat::io::IOWriter/close — the child's read-line
;; returns :None, the child returns from main, the child's stdout
;; writer drops, the parent's join handle resolves.
;;
;; Demonstrates the hologram-nesting model from arc 103a /
;; HOLOGRAM.md — neither side can reach into the other's bindings;
;; both share the binary's Rust shims (none used here beyond the
;; kernel + io + edn primitives); communication only across the
;; pipe surface.

(:wat::core::struct :demo::Ping
  (n :i64))

(:wat::core::struct :demo::Pong
  (n :i64))


;; Recursive ping-pong loop. Sends a Ping, reads the Pong, asserts
;; the n echoes correctly, logs the round, recurses with round+1
;; until round == total.
(:wat::core::define
  (:demo::ping-pong::loop
    (req-w   :wat::io::IOWriter)         ;; → child stdin
    (resp-r  :wat::io::IOReader)         ;; ← child stdout
    (out     :wat::io::IOWriter)         ;; → real OS stdout (status log)
    (round   :i64)
    (total   :i64)
    -> :())
  (:wat::core::if (:wat::core::i64::>= round total) -> :()
    ()
    (:wat::core::let*
      (((ping :demo::Ping) (:demo::Ping/new round))
       ((_send :())
        (:wat::io::IOWriter/println req-w (:wat::edn::write ping)))
       ((line :Option<String>)
        (:wat::io::IOReader/read-line resp-r))
       ((pong :demo::Pong)
        (:wat::core::match line -> :demo::Pong
          (:None     (:wat::core::panic! "ping-pong: child closed stdout early"))
          ((Some s)  (:wat::edn::read s))))
       ((n-back :i64) (:demo::Pong/n pong))
       ((_check :())
        (:wat::core::if (:wat::core::= n-back round) -> :()
          ()
          (:wat::core::panic! "ping-pong: pong n mismatch")))
       ((_log :())
        (:wat::io::IOWriter/println out
          (:wat::core::string::concat
            (:wat::core::string::concat
              "round "
              (:wat::core::i64::to-string (:wat::core::i64::+ round 1)))
            ": ping → pong"))))
      (:demo::ping-pong::loop req-w resp-r out
        (:wat::core::i64::+ round 1) total))))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((total :i64) 5)
     ;; Read the child's source from disk. read-file routes through
     ;; the cli's FsLoader, same capability gate that handles
     ;; (load!) calls in this program.
     ((child-src :String)
      (:wat::io::read-file "./wat-scripts/pong.wat"))
     ;; Spawn the child in its own frozen world. :None scope means
     ;; the child inherits this program's loader (per arc 027).
     ((proc :wat::kernel::Process)
      (:wat::kernel::spawn-program child-src :None))
     ((req-w  :wat::io::IOWriter) (:wat::kernel::Process/stdin proc))
     ((resp-r :wat::io::IOReader) (:wat::kernel::Process/stdout proc))
     ;; The conversation. Five round trips; mutual blocking on each.
     ((_loop :()) (:demo::ping-pong::loop req-w resp-r stdout 0 total))
     ;; End the conversation. Closing req-w releases the kernel
     ;; pipe write-end → child's read-line returns :None → child
     ;; exits its loop and returns from :user::main.
     ((_close :()) (:wat::io::IOWriter/close req-w))
     ;; Wait for child thread. ProgramHandle<()> joins to :();
     ;; surfaces panic-as-data via join (which raises on death —
     ;; clean exit returns unit).
     ((join-h :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::Process/join proc))
     ((_join :()) (:wat::kernel::join join-h)))
    (:wat::io::IOWriter/println stdout
      (:wat::core::string::concat
        "done — "
        (:wat::core::string::concat
          (:wat::core::i64::to-string total)
          " round trips")))))
