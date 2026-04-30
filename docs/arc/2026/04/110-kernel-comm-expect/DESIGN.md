# Arc 110 — make silent kernel-comm illegal

## Status

Drafted 2026-04-30. Replaces the originally-scoped "wrap every comm
site in expect" sweep with a substrate-level grammar rule. Arc 109
(kill-std) is paused until this lands.

## The pathology

`:wat::kernel::send` returns `:Sent ≡ :Option<()>`.
`:wat::kernel::recv` returns `:Option<T>`. Both produce `:None` when
their peer disconnects (every Sender clone dropped, or every Receiver
clone dropped). `:None` is **the terminal element of the comm
stream** — not an exception. The protocol's last message.

Today the substrate accepts code that ignores that terminal:

```scheme
((_s :wat::kernel::Sent) (:wat::kernel::send tx msg))
((_x :Option<i64>) (:wat::kernel::recv rx))
```

Both bind the comm result to a discarded name. When the peer dies
mid-flow, this thread keeps running with no signal that anything
changed. The next `recv` on a different (still-alive) channel blocks
forever, manifesting as a deadlock far from the actual fault.

Proof_004's hang was exactly this: the cache reporter panicked
(an `Atom`-on-Struct bug), the cache req-tx disconnected, the
producer's next send returned `:None`, the producer ignored it,
the producer recv'd on the still-alive ack channel — hanging.

The fix at the call site (arc 107/108) was wrapping each ignored
result with `:wat::core::option::expect`. That works one site at a
time. It does not prevent the next caller from writing the same
silent-swallow shape.

## The rule

A `:wat::kernel::send` or `:wat::kernel::recv` call may appear ONLY
in one of two syntactic positions:

1. The **discriminant** of `:wat::core::match`:
   ```scheme
   (:wat::core::match (:wat::kernel::recv rx) -> :T
     ((Some v) ...)
     (:None ...))
   ```

2. The **value-position** of `:wat::core::option::expect`:
   ```scheme
   (:wat::core::option::expect -> :T
     (:wat::kernel::send tx msg)
     "send: peer disconnected — this thread cannot survive")
   ```

Every other position — let-binding RHS, function call argument,
struct field, a function's bare return-value expression — is a
compile-time error.

The two permitted positions cover the two honest reactions to a
terminal None. `expect` is the default; `match` is the narrow case.

- **`option::expect`** — the **default**. wat is fully in-memory.
  There is no remote host, no network partition, no recoverable
  peer-disconnect. If the thing this thread is glued into dies,
  something catastrophic happened in this process; this thread
  should die too. The runtime panics with a meaningful message at
  the exact comm call site, the panic propagates up
  `join-result`, and the whole orchestrating program decides what
  to do (usually: also die).
- **`match`** — only when `:None` IS legitimate program-data. Two
  shapes earn this:
  - **Worker recv-loops** — `((Some v) recurse) (:None terminate)`.
    All client Senders have dropped; the worker exits cleanly.
    This is the `SERVICE-PROGRAMS.md` shutdown contract.
  - **Producer stages** — `((Some _) recurse) (:None ())` on a
    send. The downstream consumer dropped its Receiver; the
    producer stops producing. Stream pipelines flow this way.

A site that is neither a recv-loop nor a producer-stage is a
`expect` site. "Bind it now and decide later" was always silent
ignore in disguise; the substrate refuses to compile it.

## What the rule does NOT enforce

- It does NOT police what happens to `:Option<T>` values from other
  sources. A user who writes `(:my::lib::maybe-thing)` returning
  `:Option<i64>` may bind it to `_x` if they want to. The rule is
  scoped to `:wat::kernel::send` / `:wat::kernel::recv` because
  THOSE are the comm primitives whose `:None` carries the
  peer-death information that hangs CSP programs.
- It does NOT track comm results through helper-function returns.
  If a user writes a helper that internally does `(recv rx)` and
  returns `:Option<T>`, the helper's body must consume the recv
  via match-or-expect (the rule applies inside the helper), and
  the helper's caller treats the returned `:Option<T>` like any
  other Option. The rule is local: comm calls live where they're
  consumed.
- It does NOT touch send/recv types (they still return `:Option<()>`
  and `:Option<T>` respectively). The grammar restriction is the
  whole change.

## The four questions

**Obvious?** Yes. One grammar rule. Reading any wat file: every
`recv`/`send` is visibly attached to its consumer at the same
parenthesized form.

**Simple?** Yes. A single recursive walk over the AST tracks parent
context; comm calls in non-permitted positions push an error. No
flow analysis. No type-system extension. ~80 LOC including a new
`CheckError` variant and the walk.

**Honest?** Yes. The shape of every comm call IS the shape of every
comm-handling decision. The "bind it now and decide later" pathway
doesn't exist; the grammar refuses to compile it.

**Good UX?** Yes.
- Errors are local — "`:wat::kernel::recv` may only appear inside
  `:wat::core::match` or `:wat::core::option::expect`".
- Reading any wat file, every recv/send is visibly attached to its
  consumer at the same `(...)` form.
- Wrapping helpers stay clean: they internally `match` and return
  `T`, so the obligation gets discharged at the comm call's home file.

## Implementation

### `CheckError::CommCallOutOfPosition`

New variant in `src/check.rs::CheckError`:

```rust
CommCallOutOfPosition {
    callee: String,           // ":wat::kernel::send" or ":wat::kernel::recv"
    parent_kind: String,      // brief description of where it appeared
}
```

`Display` impl renders:

> `:wat::kernel::recv` may appear only inside `:wat::core::match`
> (as the scrutinee) or `:wat::core::option::expect` (as the
> value-position). Found in: <parent_kind>.

### `validate_comm_positions` walk

Single recursive function in `src/check.rs`:

```rust
enum CommCtx {
    /// Top-level form, function body, lambda body, struct field,
    /// call argument, let RHS — anywhere a comm call would be silent.
    Forbidden,
    /// Discriminant slot of :wat::core::match.
    MatchScrutinee,
    /// Value-position slot of :wat::core::option::expect.
    OptionExpectValue,
}

fn validate_comm_positions(
    node: &WatAST,
    ctx: CommCtx,
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, _span) = node else { return; };
    let head_str = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => {
            for child in items {
                validate_comm_positions(child, CommCtx::Forbidden, errors);
            }
            return;
        }
    };

    // 1. THIS node a comm call?
    if matches!(head_str, ":wat::kernel::send" | ":wat::kernel::recv") {
        if matches!(ctx, CommCtx::Forbidden) {
            errors.push(CheckError::CommCallOutOfPosition {
                callee: head_str.into(),
                parent_kind: "non-match-non-expect position".into(),
            });
        }
        // Recurse into children with Forbidden — comm-call args are
        // ordinary expressions; nested comm calls in them are illegal.
        for child in &items[1..] {
            validate_comm_positions(child, CommCtx::Forbidden, errors);
        }
        return;
    }

    // 2. Match dispatches its scrutinee to a permitted slot.
    if head_str == ":wat::core::match" && items.len() >= 4 {
        // Layout: (match scrut -> :T arms...)
        // items[0]=match, items[1]=scrut, items[2]="->", items[3]=:T, items[4..]=arms
        validate_comm_positions(&items[1], CommCtx::MatchScrutinee, errors);
        for child in &items[2..] {
            validate_comm_positions(child, CommCtx::Forbidden, errors);
        }
        return;
    }

    // 3. option::expect dispatches its value-position to a permitted slot.
    if head_str == ":wat::core::option::expect" && items.len() >= 5 {
        // Layout: (expect -> :T <opt> <msg>)
        // items[0]=expect, items[1]="->", items[2]=:T, items[3]=opt, items[4]=msg
        validate_comm_positions(&items[3], CommCtx::OptionExpectValue, errors);
        for (i, child) in items.iter().enumerate() {
            if i == 3 { continue; }
            validate_comm_positions(child, CommCtx::Forbidden, errors);
        }
        return;
    }

    // 4. Default — every child is a Forbidden slot.
    for child in items {
        validate_comm_positions(child, CommCtx::Forbidden, errors);
    }
}
```

### Wiring

`check_program` runs the walk over each form and each user-define's
body BEFORE the inference pass:

```rust
pub fn check_program(...) -> Result<(), CheckErrors> {
    let mut errors = Vec::new();

    for (_, func) in &sym.functions {
        validate_comm_positions(&func.body, CommCtx::Forbidden, &mut errors);
    }
    for form in forms {
        validate_comm_positions(form, CommCtx::Forbidden, &mut errors);
    }

    // ... existing inference work ...
}
```

Putting it before inference means a misplaced comm call is reported
as the structural problem it is, regardless of whether the
inference layer would have produced a follow-on type error.

## The sweep

Every existing site that doesn't fit the rule fails the new check.
The sweep migrates the codebase to comply:

### Migration default — `expect`

Default every migration to `:wat::core::option::expect`. Use
`:wat::core::match` only when this site is a true recv-loop
(`((Some v) recurse) (:None terminate)`) or a true producer-stage
(`((Some _) recurse) (:None ())`). Every other site is
expect — the in-memory peer-death is catastrophic.

### Site classes (from arc-110 inventory)

| Pattern | Sites | Action |
|---|---|---|
| `((_x :wat::kernel::Sent) (send tx msg))` | 45 | `(:wat::core::option::expect -> :() (send tx msg) "msg")`. Silent ignore was the bug; expect makes peer-death a panic-with-message. |
| `((_x :Option<T>) (recv rx))` | 11 | `(:wat::core::option::expect -> :T (recv rx) "msg")`. Same. |
| `(match (recv rx) -> :T ((Some v) recurse) (:None terminate))` worker-loop | ~14 | Already legal. No change. The terminal `:None` IS the shutdown signal. |
| `(match (send tx item) -> :() ((Some _) recurse) (:None ()))` producer-stage | ~9 | Already legal. No change. The downstream-closed `:None` stops production. |
| `((got :Option<T>) (recv rx))` then later `(match got ...)` | handful | Restructure to `(match (recv rx) -> :T arms)` at source. Removes the binding indirection. |
| `(match (recv rx) -> :T arms)` where None-arm is "panic" or "ignore" | a few | Convert to `(:wat::core::option::expect -> :T (recv rx) "msg")`. expect makes the panic intent explicit. |

### Files affected

**wat-rs:**
- `wat/std/service/Console.wat` (5 sites)
- `wat/std/stream.wat` (~9 sites — most are inside match already)
- `wat-tests/std/service-template.wat` (10 sites — the canonical template)
- `wat-tests/std/stream.wat` (~9 sites)

**lab:**
- `wat/services/treasury.wat` (~5 sites)
- `wat-tests/cache/L2-spawn.wat` (~6 sites)
- `wat-tests-integ/proof/004-cache-telemetry/*.wat` (already migrated to expect — clean)
- `wat-tests-integ/experiment/008-treasury-program/*.wat` (~75 sites — biggest concentration)

### Doc updates

- `docs/SERVICE-PROGRAMS.md` — Step 3 / 4 / 6 / 7 examples rewrite
  let-bind-recv shapes to match-at-source. Step 3's `((_s1 :Sent)
  (:wat::kernel::send tx 10))` becomes `(:wat::core::option::expect
  -> :() (:wat::kernel::send tx 10) "...")`.
- `docs/USER-GUIDE.md` § 7 Concurrency — add the rule.
- `docs/arc/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — new row.

## Slicing

| Slice | Work |
|---|---|
| **1** | Substrate change: new CheckError variant + walk + wiring + a focused unit test (deliberately-broken wat string fails the check). |
| **2** | Substrate sweep: wat-rs's own wat files + wat-tests files until cargo test green. |
| **3** | Doc sweep: SERVICE-PROGRAMS.md, USER-GUIDE.md examples comply with the rule. |
| **4** | Lab sweep: holon-lab-trading's wat / wat-tests / wat-tests-integ until cargo test green there too. |
| **5** | INSCRIPTION + 058 row + commit closure. |

Each slice ends green. The sweep is mechanical once slice 1 is in;
the compiler tells you exactly what to fix.

## Cross-references

- `docs/arc/2026/04/107-option-result-expect/INSCRIPTION.md` — the
  bridge tool's first shape.
- `docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`
  — explicitly defers the broader sweep.
- `docs/arc/2026/04/109-kill-std/REALIZATIONS.md` § "The expect
  tooling is a bridge" — names this arc as the deferred work.
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — why worker
  recv-loops legitimately exit on `:None` (terminal data, not
  exception).
- `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`
  — the deadlock that motivated arcs 107/108/110.
