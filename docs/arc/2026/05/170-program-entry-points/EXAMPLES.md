# Arc 170 — Examples

Concrete user-facing demonstrations of arc 170's canonical
forms. The principle (user direction 2026-05-10):

> *"speak in wat, not pseudo wat — wat is a communication protocol"*

Wire data is line-delimited wat values; handler bodies are real
wat with real user logic; substrate behavior is traced in wat
with comment-result style.

These examples land in arc 170 slice 5 closure paperwork
(USER-GUIDE) and seed future user-onboarding documentation.

---

## Example 1 — square server + client

A complete client/server pair. The client spawns the server,
sends numbers, reads squares, signals graceful done, and reaps
the child.

### server.wat

```scheme
(:wat::core::defn :my::handler
  [req <- :wat::holon::Atom]
  -> :wat::holon::Atom
  (:wat::core::let
    [n  (:wat::core::eval req)
     n2 (:wat::core::* n n)]
    (:wat::holon::Atom n2)))

(:wat::kernel::main! :my::handler)
```

### client.wat

```scheme
(:wat::core::defn :my::handler
  [req <- :wat::holon::Atom]
  -> :wat::holon::Atom
  (:wat::core::let
    [n  (:wat::core::eval req)
     n2 (:wat::core::* n n)]
    (:wat::holon::Atom n2)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [server (:wat::kernel::spawn-process
              (:wat::core::fn [] -> :wat::core::nil
                (:wat::kernel::server-loop :my::handler)))]

    (:wat::kernel::Process/println server (:wat::holon::Atom 3))
    (:wat::kernel::Process/readln server)
    ;; → (:wat::core::Some 9)

    (:wat::kernel::Process/println server (:wat::holon::Atom 7))
    (:wat::kernel::Process/readln server)
    ;; → (:wat::core::Some 49)

    (:wat::kernel::Process/println server :wat::core::nil)
    (:wat::kernel::Process/readln server)
    ;; → (:wat::core::Some :wat::core::nil)

    (:wat::kernel::Process/wait server)

    :wat::core::nil))
```

The client defines the same handler so the spawn-process closure
extraction can package it for the forked child. (Alternative: a
shared `handler-lib.wat` loaded by both sides.)

### Wire (line-delimited wat values; alternating sides)

```
client → server:    3
server → client:    9
client → server:    7
server → client:    49
client → server:    :wat::core::nil
server → client:    :wat::core::nil
```

The client's reads/writes go through the substrate's `Process`
pipes (parent's view of the child's fd 0 / fd 1). The server's
reads/writes go through its own ambient stdin/stdout (its fd 0 /
fd 1, which the substrate's pipe machinery connects to the
parent's `Process` handle).

### Execution trace, named in wat

**Server side:**

```scheme
;; substrate boots: StdIn / StdOut / StdErr services + per-thread Client
;; thread-locals; main runs:
(:wat::kernel::server-loop :my::handler)

;; iter 1 — client wrote 3 to server's fd 0:
(:wat::kernel::readln)                              ;; → (:wat::core::Some 3)
(:my::handler (:wat::holon::Atom 3))                ;; → (:wat::holon::Atom 9)
(:wat::kernel::println (:wat::holon::Atom 9))       ;; writes "9\n"

;; iter 2 — client wrote 7:
(:wat::kernel::readln)                              ;; → (:wat::core::Some 7)
(:my::handler (:wat::holon::Atom 7))                ;; → (:wat::holon::Atom 49)
(:wat::kernel::println (:wat::holon::Atom 49))      ;; writes "49\n"

;; iter 3 — client wrote :wat::core::nil:
(:wat::kernel::readln)                              ;; → (:wat::core::Some :wat::core::nil)
;; server-loop matches Some(:nil); returns :wat::core::nil

;; :user::main returns :wat::core::nil
;; substrate epilogue:
(:wat::kernel::println :wat::core::nil)             ;; writes ":wat::core::nil\n"
;; close fd 1; libc::exit(0)
```

**Client side:**

```scheme
;; spawn the server; substrate forks; the fn body runs as the child
(:wat::kernel::spawn-process ...)                   ;; → server :wat::kernel::Process

;; round 1
(:wat::kernel::Process/println server (:wat::holon::Atom 3))   ;; → :wat::core::nil
(:wat::kernel::Process/readln server)                          ;; blocks; returns (:Some 9)

;; round 2
(:wat::kernel::Process/println server (:wat::holon::Atom 7))   ;; → :wat::core::nil
(:wat::kernel::Process/readln server)                          ;; → (:Some 49)

;; round 3 — graceful shutdown handshake
(:wat::kernel::Process/println server :wat::core::nil)         ;; "I'm done sending"
(:wat::kernel::Process/readln server)                          ;; → (:Some :nil) — server's epilogue

;; reap the child process; collects exit status
(:wat::kernel::Process/wait server)                            ;; → :wat::core::nil

;; client's :user::main returns :wat::core::nil
;; substrate epilogue: writes ":wat::core::nil\n" to client's fd 1; libc::exit(0)
```

The graceful shutdown is symmetric:
- Client sends `:wat::core::nil` to signal "I have no more requests"
- Server's `readln` returns `Some(:nil)`; server-loop returns;
  substrate epilogue writes `:wat::core::nil` to fd 1 + exits 0
- Client's `readln` returns `Some(:nil)`; client knows the server
  exited gracefully
- Client's `Process/wait` reaps the child
- Client returns nil from main; substrate epilogue completes the
  cascade

Three layers of "I'm done" cascade outward: client → server
(graceful done), server → client (graceful epilogue), client →
its own consumer (also graceful epilogue).

---

## Example 2 — greeter server (quasiquote pattern matching)

A handler that pattern-matches on the request's AST shape using
quasiquote patterns (per arc 091 slice 8 + arc 098 pattern
grammar).

### greeter-server.wat

```scheme
(:wat::core::defn :my::handler
  [req <- :wat::holon::Atom]
  -> :wat::holon::Atom
  (:wat::core::match req
    -> :wat::holon::Atom
    (`(:greet ~name)   (:wat::holon::Atom `(:hello ~name)))
    (`(:bye ~name)     (:wat::holon::Atom `(:goodbye ~name)))
    (_                 (:wat::holon::Atom `(:unknown ~req)))))

(:wat::kernel::main! :my::handler)
```

### Wire (with a peer that drives `(:greet "Alice")` etc.)

```
peer → server:    (:greet "Alice")
server → peer:    (:hello "Alice")
peer → server:    (:bye "Bob")
server → peer:    (:goodbye "Bob")
peer → server:    (:weather "sunny")
server → peer:    (:unknown (:weather "sunny"))
peer → server:    :wat::core::nil
server → peer:    :wat::core::nil
```

`` `(:greet ~name) `` matches a 2-element AST tagged `:greet`
and binds the second element to `name`. The response is
constructed with the same quasiquote/unquote shape — wat building
wat. The protocol is wat all the way down.

---

## The motions, named

| Step | Code | What it does |
|---|---|---|
| 1. Receive a datum | `(:wat::kernel::readln)` | Substrate reads bytes up to `\n` from fd 0; parses EDN to `:wat::holon::Atom`; returns `:wat::core::Some atom` |
| 2. Process | `(handler req)` | User code; takes Atom; returns Atom |
| 3. Send response | `(:wat::kernel::println resp)` | Substrate serializes Atom to EDN; writes to fd 1 + `\n` |
| 4. Loop | `(:wat::kernel::server-loop handler)` | Tail call; arc 003 trampoline |
| 5. Peer signals done | `Some(:wat::core::nil)` from readln | Loop returns nil |
| 6. Main returns nil | implicit | Handler chain unwinds; main returns nil |
| 7. Substrate exits gracefully | substrate slice 1i epilogue | Emit `:wat::core::nil` to fd 1 → close fd 1 → libc::exit(0) |

The user writes step 2 (the handler). Everything else is
substrate. The substrate keeps the protocol; the user keeps the
logic.

## The `:nil` round-trip in the protocol

- Receiving `Some(:wat::core::nil)` ← peer told us they're done
- Returning `:wat::core::nil` from main ← we tell the substrate
  WE'RE done
- Substrate emits `:wat::core::nil` on fd 1 ← we tell the next-tier
  consumer (parent process / shell) we exited cleanly
- Substrate `libc::exit(0)` ← OS-level confirmation

Three layers of "I'm done" — peer-to-us via readln Some(:nil);
us-to-substrate via main returning nil; substrate-to-peer via
println :nil + exit 0. Each layer's "done" cascades to the next.

---

## API status note

The user-facing verbs shown in the client side
(`:wat::kernel::Process/println`, `:wat::kernel::Process/readln`,
`:wat::kernel::Process/wait`) are the proposed Type/verb shape on
the parent's `:wat::kernel::Process` value. They mirror the bare
ambient `println`/`readln` (which target the current thread's
stdin/stdout) but route through a specific spawned child's
pipes. The exact verb names are subject to refinement when slice
1h ships; the SHAPE (Type/verb on Process; mirroring the
ambient verbs) is settled per pass-12 + pass-13 doctrine.
