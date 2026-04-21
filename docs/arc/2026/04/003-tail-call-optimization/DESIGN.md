# Tail-Call Optimization in the wat Evaluator — Design

**Status:** planned. Not yet implemented.
**Motivation:** long-running driver loops written in tail-recursive
form (`:wat::std::service::Console/loop`,
`:wat::std::service::Cache/loop-step`, any future `gen_server`-shaped
wat program) currently consume one Rust stack frame per recursive
call. A Console driver processing 10k messages burns 10k frames; at
the default 8MB thread stack this eventually overflows.

The wat source is already written in tail-recursive shape. The
evaluator needs to recognize it.

---

## The problem, shown

`wat/std/service/Console.wat`, simplified:

```scheme
(:wat::core::define
  (:wat::std::service::Console/loop (rxs ...) (stdout ...) (stderr ...) -> :())
  (:wat::core::if (:wat::core::empty? rxs) -> :()
    ()
    (:wat::core::let* (...)
      (:wat::core::match maybe -> :()
        ((Some tagged)
          (:wat::core::let* (...)
            (:wat::std::service::Console/loop rxs stdout stderr)))    ; ← tail call
        (:None
          (:wat::std::service::Console/loop
            (:wat::std::list::remove-at rxs idx)
            stdout stderr))))))                                       ; ← tail call
```

The recursive `Console/loop` invocation is the **last thing** each
match branch does. Nothing wraps the call's return value; nothing
uses it; the enclosing frame's only remaining action is to return
whatever the recursive call returned. **Textbook tail position.**

Today's `runtime::apply_function`:

```rust
pub fn apply_function(
    func: &Function,
    args: Vec<Value>,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // ... arity check, param binding, env construction ...
    let call_env = builder.build();
    match eval(&func.body, &call_env, sym) {
        Err(RuntimeError::TryPropagate(e)) => Ok(Value::Result(Arc::new(Err(e)))),
        other => other,
    }
}
```

`eval(&func.body, ...)` is an ordinary Rust function call. Each wat
function invocation adds one Rust stack frame. The `Console/loop`
driver calling itself in tail position creates a fresh `apply_function`
frame AND a fresh `eval` frame per message, linearly, until either
the thread dies or the process runs out of stack.

No TCO today. `Grep runtime.rs "tail"` returns one comment about an
unrelated type-signature tail.

---

## Path A — trampoline TCO in the evaluator

### The approach

Standard interpreter trampoline:

1. Introduce a control-flow signal, say `RuntimeError::TailCall { func, args }`.
   Treated the same way `TryPropagate` is — a hidden internal
   variant, never surfacing to user code, caught at a fixed catch point.
2. `apply_function` wraps its evaluation in a `loop { ... }`. Each
   iteration binds a fresh env for `func`'s params and evaluates
   the body.
3. When the evaluator is in **tail position** and about to invoke a
   user-defined function, it evaluates the args, then returns
   `Err(RuntimeError::TailCall { new_func, new_args })` instead of
   recursing into `apply_function`.
4. `apply_function`'s loop catches `TailCall`, reassigns `func` and
   `args`, loops again. **No Rust stack growth.**

```rust
pub fn apply_function(
    func: &Function,
    args: Vec<Value>,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // ... arity check for the initial call ...
    let mut current_func: Arc<Function> = /* the entry func */;
    let mut current_args: Vec<Value> = args;
    loop {
        // Rebind params in a fresh child of current_func's closed env.
        let call_env = /* ... build from current_func and current_args ... */;
        match eval_in_tail_position(&current_func.body, &call_env, sym) {
            Err(RuntimeError::TailCall { func: f, args: a }) => {
                current_func = f;
                current_args = a;
                continue;
            }
            Err(RuntimeError::TryPropagate(e)) => {
                return Ok(Value::Result(Arc::new(Err(e))));
            }
            other => return other,
        }
    }
}
```

### What "tail position" means mechanically

Tail position is a property of an AST node **with respect to an
enclosing function**. The rule: a sub-expression is in tail position
if its value will BE the enclosing function's return value without
further computation.

Positions that carry tail-position through:

| Form | Which sub-position is tail |
|---|---|
| `(:wat::core::define (name ...) body)` | `body` |
| `(:wat::core::lambda (...) body)` | `body` |
| `(:wat::core::let ((...) rhs) body)` | `body` (the RHS is NOT tail) |
| `(:wat::core::let* ((b1 rhs1) (b2 rhs2) ...) body)` | `body` (every RHS is NOT tail) |
| `(:wat::core::if cond -> :T then else)` | `then` and `else` (cond NOT tail) |
| `(:wat::core::match scrutinee -> :T (pat1 body1) ...)` | every `body_i` (scrutinee NOT tail) |

(`:wat::core::when` is listed in FOUNDATION as a host-inherited
Lisp form; NOT shipped in wat-rs today. When it ships, its body
is tail-carrying — cond is not.)

Positions that are **never tail**:

- Arguments being evaluated before a function call — they feed into
  the call; they're not the return value.
- Operands to primitives: `(:wat::core::+ a b)`, `(:wat::algebra::Bundle xs)`,
  `(:wat::core::vec a b c)` — the primitive does work after
  evaluating the children.
- The RHS of any `let` / `let*` binding (except by being the last
  expression in the body, which isn't a binding).
- The inner expression of a constructor wrapper: `(Ok expr)`,
  `(Err expr)`, `(Some expr)` — the constructor still has to wrap
  the result. The **whole** `(Ok <call>)` may be in tail position,
  but the inner `<call>` is not.

### Which calls get TCO'd

Only **user-defined function calls** — entries in the `SymbolTable`.
Primitives (`:wat::core::*`, `:wat::algebra::*`, `:wat::kernel::*`,
auto-generated struct accessors, `#[wat_dispatch]` shims) do not go
through `apply_function`; they dispatch directly. A primitive in
tail position runs normally — no TailCall signal, no loop
continuation. Its return value is the function's return value, same
as any other expression.

**Lambda calls** can also be TCO'd. `apply_function` takes a `Function`
which carries `closed_env: Option<Environment>`. When the signal's
`func` is a lambda, the rebind uses the lambda's closed env as the
parent; when it's a define, the parent is a fresh root. Same
machinery either way.

### Self, mutual, and foreign tail calls

**Self-recursion** — `Console/loop` calling itself. The common case.
Works automatically: the signal carries the new func, the loop
reassigns, stack stays constant.

**Mutual recursion** — `A` tail-calls `B`, `B` tail-calls `A`. Also
automatic: the signal carries whichever func was named, the loop
reassigns. `apply_function`'s loop doesn't care which function is
next; it just keeps rebinding and evaluating.

**Call into a non-wat function** (kernel primitive, Rust shim,
auto-accessor) — the signal never fires because those don't go through
`apply_function`. The call's result feeds back into the current
body's evaluation.

### What's NOT optimized

**Non-tail recursion.** `(:wat::core::+ 1 (recurse ...))` still burns a
stack frame because `+` has to wait for the recursive call's result.
This is correct — the frame is load-bearing; there's information
in it that the continuation needs. Authors writing deep non-tail
recursion continue to face the default-stack limit. The fix for
that is different (grow the stack — see Path B below — or restructure
the algorithm).

**Recursion inside a Result constructor.** `(Ok (recurse ...))` is
NOT tail — the `Ok` wraps the recursive call's value. Authors who
want TCO on a Result-returning function write the recursive call
directly (not wrapped in Ok/Err) and let the callee's own return
wrap it. The typical pattern:

```scheme
(:wat::core::define (:my::loop (state :S) -> :Result<T,E>)
  (:wat::core::if (done? state) -> :Result<T,E>
    (Ok (extract state))                      ; base case — NOT tail
    (:my::loop (advance state))))             ; recursive — TAIL, no Ok wrap needed
                                              ;  because :my::loop itself returns Result
```

The recursive call's Result IS the function's Result. No need to
unwrap and re-wrap.

---

## Why this is the right answer — the references

### Scheme: TCO is mandated

R5RS, R6RS, R7RS all require that tail calls do not consume
unbounded stack. This is the primary reason Scheme programs can be
written recursively without iteration constructs — `(let loop (...)
...)` IS the loop.

Tail-position specification lifted essentially verbatim into wat's
form list above. Scheme's rules:

- `lambda` body is tail
- last expression of `begin` is tail
- `if` consequent and alternative are tail
- `cond` clause bodies (after the test) are tail
- `case` match bodies are tail
- `and` / `or` — only the LAST operand is tail (because early
  short-circuit means earlier operands have a continuation)

wat doesn't ship `and`/`or` as short-circuit forms today, but the
rule would apply if it did.

### Racket: TCO is load-bearing for the module system

Racket inherits Scheme's TCO and extends it across contract
boundaries. Contract wrappers around functions preserve the
tail-call property. Not directly relevant for wat-rs but a useful
data point: real systems built on Scheme-TCO assume it works at
every layer, including their own metaprogramming.

### Erlang / BEAM: TCO is the gen_server substrate

This is the strongest reference for wat's use case.

Every Erlang server is a tail-recursive `loop/N` function. The
canonical shape:

```erlang
loop(State) ->
    receive
        {call, From, Msg} ->
            {Reply, NewState} = handle(Msg, State),
            From ! Reply,
            loop(NewState);                   % tail call
        stop ->
            ok
    end.
```

Exactly the shape wat's Console/loop has: `match` on incoming
message, do work, recurse tail. BEAM's `call_only` instruction
reuses the current function frame — no push, no return, just a
parameter rebind and re-entry.

Erlang's design makes TCO mandatory for correctness. Processes run
for the lifetime of the VM — a server that handled a million
messages over its lifetime would have a million-deep stack without
TCO. Impossible to run at scale. So BEAM does it.

wat's driver loops are structurally identical. The TCO we need is
the same TCO Erlang has been relying on since 1986.

### OCaml / Haskell: sometimes

OCaml's native compiler TCOs self-tail-calls since 4.02 (2014) and
mutual tail-calls usually. Programmers use `[@tailcall]` to force
the check. The language does NOT mandate it.

Haskell's laziness makes "tail position" weirder — WHNF evaluation
means some "loops" are actually unfolding lazy structures on the
heap. In practice GHC optimizes self-tail-calls; programmers use
`foldr` / `foldl'` with strictness to control stack.

Neither informs wat-rs directly. Scheme and Erlang are the
references.

### Clojure: `recur` because the JVM won't

Clojure runs on the JVM, which doesn't support TCO. Rich Hickey
chose NOT to work around this invisibly; instead Clojure has
explicit:

- `(recur ...)` — self-tail-call; compiler error if not in tail
  position. Stack stays flat for self-recursion only.
- `(trampoline f ...)` — mutual-tail-call helper; each function
  returns a thunk and the trampoline keeps calling until a
  non-function value emerges.

Hickey's stance: "You should know whether you're recursing or
looping." The explicit form forces the author to think.

wat-rs could take the Clojure path (explicit `recur` form, no
implicit TCO), but the wat code as written today doesn't use an
explicit form — `Console/loop` just names itself at tail position.
Going the Clojure route would mean either (a) adding `recur` as a
new form and rewriting the stdlib loops, or (b) keeping what wat
already has and making the interpreter do the work. (b) is what
Scheme and Erlang chose; (b) is what the wat source already expects.

### Rust itself: no

Rust has no TCO. Workarounds: manual `while let`, the `stacker`
crate for stack growth, or the `trampoline` crate pattern. wat-rs
implements its own in the evaluator because the language we're
hosting (wat) needs it; the host language's lack of it isn't a
constraint on the interpreter's semantics.

---

## Path B — grow the Rust stack on demand

The `stacker` crate provides `stacker::maybe_grow(red_zone,
stack_size, callback)` — if the current thread's stack has less
than `red_zone` remaining, allocate a new `stack_size` chunk and
run `callback` on it. Classic Rust TCO workaround.

Dropping `stacker::maybe_grow(64 * 1024, 4 * 1024 * 1024, || { eval(...) })`
around the recursive `eval` call in `apply_function` gives each wat
function effectively unlimited stack. No evaluator rewrite. Works
for non-tail recursion too.

**Why this is a band-aid, not the answer:**

1. It treats a language-level constraint (tail position) as a
   resource-management problem (stack size). The wat author writing
   a tail-recursive driver should get constant-stack execution as
   a **language guarantee**, not as "the implementation happened
   to grow the stack fast enough."
2. Memory consumption is still linear in recursion depth. A driver
   that handles 10M messages uses 10M frames' worth of memory
   before anything GCs it. For long-running services that's a
   resource leak.
3. It muddies the contract. Authors who write non-tail recursion
   get it to "work" and never learn that they should have
   restructured. The lack of crash hides the design issue.
4. Every other Lisp-family language that runs indefinite processes
   on top of a managed runtime solved this at the interpreter
   level. Following that tradition is right.

Path B can land as a stopgap — a single `stacker::maybe_grow`
around the `eval(body)` call buys us immediate headroom while Path
A is built. But the real answer stays Path A.

---

## Implementation sketch

### Phase 1 — add the signal

New variant on `RuntimeError`:

```rust
/// Internal tail-call signal — raised when the evaluator recognizes
/// a user-function call in tail position. Carries the next function
/// and its already-evaluated args up to `apply_function`'s loop,
/// which reassigns and re-iterates without growing the Rust stack.
///
/// Like `TryPropagate`, never surfaces to user code; `apply_function`
/// is the only catch site. Checker has no say in it (tail position
/// is a runtime property of the evaluator, not a type-system property).
TailCall {
    func: Arc<Function>,
    args: Vec<Value>,
},
```

### Phase 2 — thread tail-position through eval

`eval` gains a sibling `eval_tail`, OR `eval` gains a `tail: bool`
parameter. Pattern match varies:

- On a `WatAST::List` whose head is `:wat::core::if`: evaluate cond
  non-tail; dispatch to then/else, passing `tail = self.tail`.
- On `:wat::core::let` / `let*`: evaluate bindings' RHS non-tail;
  evaluate body with `tail = self.tail`.
- On `:wat::core::match`: evaluate scrutinee non-tail; evaluate each
  arm body with `tail = self.tail`.
- On a user-function call (head resolves in `sym.functions`):
  - If `tail`: evaluate args (non-tail), then return
    `Err(TailCall { func: looked_up, args: evaluated })`.
  - If not tail: evaluate args, call `apply_function(...)`.
- On a primitive: evaluate children non-tail, run the primitive.
  The primitive's return value carries the `tail` status through
  — if the whole primitive call was in tail position, its return
  IS the function's return, same as any value.

### Phase 3 — wrap apply_function in a loop

Rewrite the single-shot `eval(body)` into a `loop` that catches
`TailCall` and reassigns:

```rust
pub fn apply_function(
    func: &Function,
    args: Vec<Value>,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let mut cur_func: Arc<Function> = /* Arc over the passed func */;
    let mut cur_args: Vec<Value> = args;
    loop {
        // Arity + param binding per iteration
        if cur_args.len() != cur_func.params.len() {
            return Err(RuntimeError::ArityMismatch { /* ... */ });
        }
        let parent = cur_func.closed_env.clone().unwrap_or_default();
        let mut builder = parent.child();
        for (name, value) in cur_func.params.iter().zip(cur_args.drain(..)) {
            builder = builder.bind(name.clone(), value);
        }
        let call_env = builder.build();
        match eval_in_tail(&cur_func.body, &call_env, sym) {
            Ok(v) => return Ok(v),
            Err(RuntimeError::TailCall { func: f, args: a }) => {
                cur_func = f;
                cur_args = a;
                continue;
            }
            Err(RuntimeError::TryPropagate(e)) => {
                return Ok(Value::Result(Arc::new(Err(e))));
            }
            Err(other) => return Err(other),
        }
    }
}
```

### Phase 4 — tests

**Self-recursion depth test.** Write a wat function that recurses
on itself N times in tail position, where N > default stack frame
budget (say 1_000_000). Verify it runs without overflow and returns
the correct value.

```scheme
(:wat::core::define (:app::loop (n :i64) (acc :i64) -> :i64)
  (:wat::core::if (:wat::core::= n 0) -> :i64
    acc
    (:app::loop (:wat::core::- n 1) (:wat::core::+ acc 1))))

;; main invokes (:app::loop 1000000 0) and expects 1000000
```

Without TCO: stack overflow. With TCO: runs in constant stack, any
N fits.

**Mutual-recursion test.** Two functions that call each other in
tail position.

**Console/loop stress test.** Run a Console driver that processes
N messages, N >> default stack depth. Verify completion.

**Non-tail recursion still bounded.** A function that recurses in
non-tail position continues to use stack; verify a modest depth
works, and a very deep one crashes as before. This confirms TCO
doesn't accidentally optimize cases it shouldn't.

---

## Open questions

1. **The `try` interaction.** `try` raises `TryPropagate`; a tail
   call raises `TailCall`. Both are internal signals caught at
   `apply_function`. If a tail-called function's body raises
   `TryPropagate`, the catching `apply_function` must handle it —
   the loop iteration has already begun; we need to exit with the
   propagated Err. The sketch above handles it (the `TryPropagate`
   arm returns immediately). Confirm in implementation that no
   subtle interleaving breaks the forcing function.

2. **Mutual tail calls across Result-returning functions.** If `A`
   returns `:Result<T,E>` and tail-calls `B` which also returns
   `:Result<T,E>`, the TCO machinery applies. If `A` returns `:i64`
   and somehow tail-calls a Result-returning `B`... type check
   should refuse this at startup. Confirm the checker catches it.

3. **Stack probe for non-tail recursion.** When the sketch is in
   place, do we ALSO add a `stacker::maybe_grow` for non-tail
   recursion as an insurance policy? The trading lab's rhythm code
   is mostly tail-recursive, but deep non-tail stuff (if any
   emerges) would still benefit. Decide after Path A ships — don't
   mix concerns.

4. **Observability.** Should the evaluator emit a trace when TCO
   fires? Useful during development (confirms the optimization is
   actually running); noisy in production. Probably a build-time
   feature flag, off by default.

5. **Lambda tail calls across closure boundaries.** A lambda
   captures its enclosing env. When the lambda is tail-called and
   reassigned into `apply_function`'s loop, the next iteration's
   `call_env` parent is the lambda's `closed_env`. Confirm this is
   what the existing machinery does — it should "just work" since
   the signal carries the full `Function` including `closed_env`.

---

## References

- **R7RS §3.5 Proper tail recursion** —
  https://small.r7rs.org/attachment/r7rs.pdf (page 18)
- **Erlang BEAM `call_only` instruction** — Joe Armstrong's thesis,
  Chapter 4; BEAM reference manual
- **Clojure's `recur` and `trampoline`** — Hickey's "Clojure for
  Lisp Programmers" talk; Clojure.org documentation
- **The `stacker` crate** — https://crates.io/crates/stacker
- **Cheney-on-the-MTA** (Chicken Scheme's approach) — Baker 1994
  "CONS Should Not CONS Its Arguments, Part II: Cheney on the M.T.A."

The wat-rs implementation follows Scheme + Erlang: interpret tail
position at evaluation time, trampoline through `apply_function`.
Explicit `recur` form (Clojure style) rejected because the wat
source is already written in tail-recursive shape without it; the
interpreter should honor what the author wrote.

---

## The payoff

`Console/loop`, `Cache/loop-step`, and every future `gen_server`-
shaped driver becomes structurally stack-free. The wat code we
already wrote honors the shape it always intended to have. Long-
running drivers run indefinitely without stack growth. The language
joins the tradition of Lisps that take tail calls seriously because
the programs written in them demand it.

*these are very good thoughts.*

**PERSEVERARE.**
