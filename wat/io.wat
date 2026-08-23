;; wat/io.wat — wat-level IO conveniences over the Rust IOWriter primitives.
;;
;; THE `with-` NAMING LAW (declared 2026-06-10): `with-` means MANAGED SCOPE — the framework
;; owns creation + destruction, the caller owns only usage (Clojure's with-open sense). A form
;; where the caller manages the lifecycle itself does NOT get `with-`.
;;
;; The opt-in ladder for file writing:
;;   write-file        — one-shot: hand over path + content, we do everything, no handle surfaced.
;;   with-open-file    — managed scope: we hand you the writer, you use it, we close it (RAII on error).
;;   IOWriter/open-file + close — explicit: you own the handle and the close (already live, Rust).
;;
;; Arc 170 stdio-as-defservice — the raw-fd constructors (Rust builtins, kernel-restricted):
;;   (:wat::io::IOWriter/from-fd fd) -> :wat::io::IOWriter   ;; #[restricted_to :wat::kernel::]
;;   (:wat::io::IOReader/from-fd fd) -> :wat::io::IOReader   ;; #[restricted_to :wat::kernel::]
;; DUP-then-own: each dup(2)s the caller's fd and owns ONLY the dup (Drop closes the dup, never the
;; real fd 0/1/2). Privileged (forging a handle from a raw fd is a capability) — only kernel-internal
;; wat may call them, e.g. the primed stdio defservices' generated `::init` in
;; wat/kernel/services/stdio.wat. The fd is a pure i64 that rides `Admin::Init` clean; the
;; impure handle is BORN inside init, never passed as an init param (arc 293.W Pure-Admin wall).

;; read-file — Ruby's File.read. Opens a file at `path`, reads the whole content to a
;; String (byte-faithful UTF-8 decode), and the reader's Arc drops at scope end so RAII
;; (Drop) closes the fd. The read mirror of write-file: one-shot, no handle surfaced.
(:wat::core::defn :wat::io::read-file [path <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [r (:wat::io::IOReader/open-file path)]
    (:wat::io::IOReader/read-all-string r)))

;; write-file — Ruby's File.write. Opens, writes the whole content, closes. Surfaces NO handle,
;; so there is nothing for the caller to leak; on a mid-write error the writer's Arc drops and
;; RAII (Drop) closes the fd. NOT a `with-` form: there is no scope handed to the caller.
(:wat::core::defn :wat::io::write-file [path <- :wat::core::String content <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [w (:wat::io::IOWriter/open-file path)]
    (:wat::core::do
      (:wat::io::IOWriter/write-string w content)
      (:wat::io::IOWriter/close w))))

;; with-open-file — Ruby's `File.open(path) do |w| … end`. Opens a writer, hands it to body-fn,
;; closes it after (explicitly on success; via RAII Drop if body-fn errors and the scope unwinds).
;; Returns body-fn's result. The `with-` earns its meaning: managed scope, caller owns only usage.
(:wat::core::defn :wat::io::with-open-file :- [T] [path <- :wat::core::String body-fn <- [:wat::io::IOWriter :-> T]] -> :T
  (:wat::core::let [w      (:wat::io::IOWriter/open-file path)
                    result (body-fn w)]
    (:wat::core::do
      (:wat::io::IOWriter/close w)
      result)))
