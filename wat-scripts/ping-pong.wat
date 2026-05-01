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


;; Recursive ping-pong loop. Sends a Ping via process-send, reads the
;; Pong via process-recv, asserts the n echoes correctly, logs the
;; round, recurses with round+1 until round == total.
(:wat::core::define
  (:demo::ping-pong::loop
    (proc    :wat::kernel::Process<demo::Ping,demo::Pong>)
    (out     :wat::io::IOWriter)         ;; → real OS stdout (status log)
    (round   :i64)
    (total   :i64)
    -> :())
  (:wat::core::if (:wat::core::i64::>= round total) -> :()
    ()
    (:wat::core::let*
      (((ping :demo::Ping) (:demo::Ping/new round))
       ((_send :())
        (:wat::core::result::expect -> :()
          (:wat::kernel::process-send proc ping)
          "ping-pong: send to child failed"))
       ((pong :demo::Pong)
        (:wat::core::match (:wat::kernel::process-recv proc) -> :demo::Pong
          ((Ok (:wat::core::Some v)) v)
          ((Ok :wat::core::None)
           (:wat::core::panic! "ping-pong: child closed stdout early"))
          ((Err _died)
           (:wat::core::panic! "ping-pong: child died"))))
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
      (:demo::ping-pong::loop proc out
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
     ;; Arc 105a: spawn-program returns Result; pattern-match
     ;; immediately because :user::main's `-> :()` signature can't
     ;; propagate Err via `:wat::core::try`. A real failure here
     ;; means the embedded child source has a startup error —
     ;; demo author's bug, panic is the right surface.
     ((proc :wat::kernel::Process<demo::Ping,demo::Pong>)
      (:wat::core::match (:wat::kernel::spawn-program child-src :wat::core::None)
        -> :wat::kernel::Process<demo::Ping,demo::Pong>
        ((Ok p) p)
        ((Err err)
         (:wat::core::panic!
           (:wat::core::string::concat
             "ping-pong: spawn failed: "
             (:wat::kernel::StartupError/message err))))))
     ;; The conversation. Five round trips; mutual blocking on each.
     ((_loop :()) (:demo::ping-pong::loop proc stdout 0 total))
     ;; End the conversation. Closing the child's stdin via the
     ;; Process stdin accessor releases the kernel pipe write-end
     ;; → child's read-line returns :None → child exits its loop
     ;; and returns from :user::main.
     ((_close :()) (:wat::io::IOWriter/close (:wat::kernel::Process/stdin proc)))
     ;; Wait for child thread via Process/join-result.
     ((_wait :())
      (:wat::core::match (:wat::kernel::Process/join-result proc) -> :()
        ((Ok _) ())
        ((Err _died)
         (:wat::core::panic! "ping-pong: child died unexpectedly")))))
    (:wat::io::IOWriter/println stdout
      (:wat::core::string::concat
        "done — "
        (:wat::core::string::concat
          (:wat::core::i64::to-string total)
          " round trips")))))
