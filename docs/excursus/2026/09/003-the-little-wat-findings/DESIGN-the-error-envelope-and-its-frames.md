# DESIGN — the error envelope and its frames

**Drawn 2026-09-25, for the builder's ruling before anything is built.** Grounded in two read-only
reports committed at `5459ecc3c`: `AUDIT-error-shapes-and-backtraces.md` (the measurement map) and
`reports/conformare.md` (the ward). Every claim below was re-checked against the code by the
orchestrator.

Builder: *"i've been extremely skeptical we actually built the backtraces i wanted. even the error
shapes.. i think they are suspect... we may still be shipping stringified structure causing double
quoting which reads very poorly -- and isn't processable as the structure is lost"*. And the intent:
*"we added rust locations in our backtraces to provide the full path of the backtrace … so we could
debug our own code through whole stack - kinda like how clojure has java in its traces"*, with a
span's end an `Option` *"as only wat knows where some expr ends but both wat and rust know where expr
begins"*.

## What is true today (measured)

1. **The Rust error types are sound.** They follow Pattern A (`docs/CONFORMARE.md`, arc 243): an outer
   struct with a mandatory location. That half of arc 296 held.
2. **The wat envelope is not.** `:wat::kernel::LociDiedError` (`wat/kernel/diagnostics.wat:122`)
   declares **5 of 7 variants as a bare `String`** — `Panic.message`, `RuntimeError`,
   `EntryFormFailure`, `MainSignature`, `BadReturn`. A structured error crossing one of them is
   serialized INTO that string (`src/process/died.rs`). Every RuntimeError in the 364-probe corpus
   arrives double-quoted; some three levels deep. The other flattening source is one line,
   `src/macros/error_edn.rs:157` (`Validator(e) => first_line(e.to_string())`).
3. **Three shapes mean "a location":**

   | record | fields | end |
   |---|---|---|
   | `:wat::core::Span` — every runtime `:location` | `file line col end` | `Option` |
   | `:wat::kernel::Location` — in the `:wat::core::Error` surface, `Fault`, `Failure` | `file line col` | none |
   | `:wat::kernel::Frame` — the only backtrace shape | `file line symbol` | no col, no end |

4. **The backtrace exists and is thrown away.** `CALL_STACK` (`src/value/frame.rs:35`) holds a
   `FrameInfo { callee_path, call_span: Span }` per call, pushed in `apply_function`
   (`src/runtime.rs:~11137`) and LIVE when a `RuntimeError` is raised. `RuntimeError::new` never reads
   it; `LociDiedError.RuntimeError` has nowhere to put it. Only assertion panics carry `:frames`
   (a separate path, `AssertionPayload` → `panic_any`). `std::backtrace::Backtrace`: zero uses.
5. **Arc 296's close is unsupported.** Its DESIGN points at an `INSCRIPTION.md` that does not exist; the
   payload-to-structure claim holds for 1 of 6 variants; arc 278's *"Zero string-wrapping remains"* is
   false.

## The proposal — four decisions

### D1 — ONE location shape: `:wat::core::Span`

`{file line col end: (Option :- [Pos])}` everywhere a location appears — the `:wat::core::Error`
surface's `location`, `Fault`, `Failure`, and every frame. `:wat::kernel::Location` retires (or becomes
an alias). A Rust-originated location is a `Span` with `end` `None`; a wat one has `Some`. That is
exactly the builder's stated distinction, and it is already the shape of every runtime `:location`.

| | |
|---|---|
| Obvious | YES — one answer to "where", everywhere |
| Simple | YES — one record replaces three |
| Honest | YES — `end` absent means "only the start is known", which is precisely true for Rust |
| Good UX | YES — a consumer writes one location reader, not three |

### D2 — The envelope carries structure, never a string

Every `LociDiedError` variant that describes a failure carries the structured `:wat::kernel::Failure`
record — `error <- :wat::core::Error` plus `frames` (D3) — instead of `message <- String`. The
transport variants (`Disconnected`, `Stopped`) stay bare. The human headline is the `:wat::core::Error`
surface's own `message` field: a short sentence, never serialized EDN.

| | |
|---|---|
| Obvious | YES — the error IS data, from the inside out |
| Simple | YES — one failure record for every failure variant |
| Honest | YES — nothing is flattened, so nothing is lost and nothing needs re-parsing |
| Good UX | YES — the double-quoted blob disappears; a program can `match` on the cause |

### D3 — Every failure carries frames: wat frames AND the Rust frame, user first

`:wat::kernel::Frame` becomes `{symbol <- String  span <- :wat::core::Span  kind <- (:wat | :rust)}`.

- **wat frames**: `CALL_STACK` snapshotted when the error is created — one frame per active call,
  `symbol` = the callee, `span` = the call site. It is already live at that moment; the cost is a copy.
- **the Rust frame**: the `rust_caller_span!()` of the site that RAISED the error, `kind :rust`,
  `end None`. This is the "java in the clojure trace" the builder asked for, and it is where stones B,
  O and P's removed Rust origins come back — as a frame beside the user's location, not instead of it.

| | |
|---|---|
| Obvious | YES — the stack you would expect, in the order you would read it |
| Simple | YES — one capture point, one frame shape |
| Honest | YES — both halves of the story, each labelled with what it is |
| Good UX | YES — the user finds their line; the builder debugs the substrate |

⚠ **Rust depth — one frame, not a full `std::backtrace`.** A full native backtrace is expensive,
build-dependent (inlining, symbol stripping), and non-deterministic across builds — it would make every
diagnostic golden unstable. The raising site is the Rust frame that carries meaning, and it is free.
**This is the one sub-decision the builder may want differently.**

### D4 — The primary `:location` is DERIVED: the innermost frame in user source

With frames present, "where did this happen in MY program" stops being a separate field to get right at
every raise site: it is the innermost `:wat` frame whose file is not stdlib. That one rule dissolves
**C-114** (`+` overflow reported at `wat/core.wat:66`) and every future "located in our source" case.
"Stdlib" is decided by the loader's existing `Privilege::Stdlib`, **never by a filename prefix** (a
prefix is the same convention-not-shape defect this campaign keeps removing).

| | |
|---|---|
| Obvious | YES — the location is "your innermost line", always |
| Simple | YES — derived from frames, not maintained per site |
| Honest | YES — the stdlib and Rust frames are still there, one level down |
| Good UX | YES — no error ever points only into wat-rs again |

## Consequences, named

- **Every diagnostic golden changes.** The flattened blobs become structured maps; frames appear. This
  is the whole point, and it is large: recapture is a campaign step, not a side effect.
- **The wire carries the envelope** (`LociDiedError` crosses process boundaries as EDN). Frames and
  `Span` are pure records, so they cross — to be verified, not assumed.
- **Recursion depth.** A deep recursion produces a deep `CALL_STACK`. Needs a cap (e.g. innermost N +
  outermost M, with an elision count). A number to measure, not guess.
- **`first_line(e.to_string())` at `macros/error_edn.rs:157`** is replaced by the validator error's own
  structured message — the `FreezeValidatorError` trait needs a `message()`, per the ward.
- **Arc 296 gets an honest close record** once this lands: what it claimed, what held, what this fixed.

## Questions for the builder

1. **D1** — retire `:wat::kernel::Location` in favour of `:wat::core::Span` everywhere?
2. **D2** — every failure variant carries `Failure`; the string `message` fields go?
3. **D3** — the Rust side of a trace is the ONE raising site (deterministic), not a full native
   backtrace?
4. **D4** — the primary location is derived as "innermost user-source frame", with stdlib decided by
   load privilege?

## Delivery shape, once ruled

Not one strike. In order, each landing green: (1) the location shape (D1); (2) frames captured on the
Rust error types (D3) — invisible to users until (3); (3) the envelope carries `Failure` (D2), the golden
recapture; (4) the derived primary location (D4), which retires C-114.
