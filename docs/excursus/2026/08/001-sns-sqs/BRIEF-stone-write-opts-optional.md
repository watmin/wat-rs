# BRIEF — excursus 001 stone WO-OPT: the opts argument becomes OPTIONAL

**Builder's ruling 2026-08-30:**

> *"i say its optional... if you omit it, you get the defaults, if you want to change it, you
> pass the config ops you want for your call"*

**This corrects MY under-specification, not the executor's work.** Stone WRITE-OPTS shipped
`write-json` at a required arity of 2 because my BRIEF's sketch showed
`(:wat::edn::write-json v (:wat::edn::opts))` at every call site. Measured after the fact:

```
(:wat::edn::write-json v)  →  :wat::edn::write-json: expected 2 argument(s); got 1
```

That is not what was asked for. The default should apply when the argument is omitted.

## The work

`:wat::edn::write-json` and `:wat::edn::write-json-natural` accept **1 or 2 args**. One arg
means `(:wat::edn::opts)` — nanos, the sane default. Two means the caller's own opts.

## The exemplar — copy it, do not invent

`:wat::io::IOReader/read-frame` is exactly this shape and is live. Three parts:

1. **`src/intrinsic/io/reader.rs:410`** — the handler takes `xs: &[WatAST]`, i.e. **Variadic**,
   and reads its optional second element itself.
2. **`src/check.rs:9281` `infer_ioreader_read_frame`** — a named inference fn that owns the
   arity guard: `if args.is_empty() || args.len() > 2`. **The guard lives in the CHECKER**,
   where it produces a real diagnostic, not hand-rolled in the runtime.
3. **`src/check.rs:2977`** — the dispatch arm that intercepts the verb and calls that fn. Its
   own comment states the contract: *"accepts 1 arg (reader) or 2 args (reader, max-bytes).
   … the scheme in `register_io_scheme` handles the 1-arg (default) path via normal dispatch,
   but we intercept both here for uniform handling."*

`src/intrinsic/io/reader.rs:80` flags `read-frame` as the one exception of ten — **do the same
for the JSON verbs**: say in the file's header that they are the optional-arity rows, so the
next reader does not think a plain `Exact` was lost by accident.

⚠ `src/intrinsic/mod.rs:142` — *"Only `Exact` and `Variadic` are needed now — Range/AtLeast are
out of scope."* Do **not** add a `Range` arity to the registry to make this prettier. That is a
registry-shape change and it is arc 255's, not this excursus's. If Variadic-plus-checker-guard
turns out to be unworkable, STOP and report rather than reshaping the registry.

## Read in order

1. `src/intrinsic/io/reader.rs:80` and `:410` — the header note and the Variadic handler.
2. `src/check.rs:9281` (`infer_ioreader_read_frame`) and `:2977` (the dispatch arm).
3. `src/check.rs:19100`-ish — where WRITE-OPTS split the registration into 1-arg and 2-arg
   loops. The JSON verbs move out of the fixed-2 loop into this intercepted shape.
4. `src/edn/render.rs` — the two JSON handlers, currently `(v, opts)`; they become `&[WatAST]`.
5. `wat/edn.wat` — unchanged. The struct and its constructors are correct and stay.

## Blast radius

- `src/check.rs` — registration + a dispatch arm + an `infer_` fn per verb
- `src/edn/render.rs` — the two handlers take a slice
- `src/intrinsic/edn.rs` — arity declaration for the two verbs
- `wat-tests/edn/write-opts.wat` — add the 1-arg rows (see the gate)
- optionally: revert the 8 live call sites to the 1-arg form where they want the default
- this excursus's SCORE

**`wat/edn.wat`, `crates/wat-edn/`, and `:wat::edn::write` are NOT touched.**

## The gate

Extend `wat-tests/edn/write-opts.wat` — it is yours, it already tests the 2-arg path. Add:

- `(write-json v)` type-checks and equals `(write-json v (:wat::edn::opts))` **exactly**
- same for `write-json-natural`
- 3 args is still a type error (the guard's upper bound holds)
- 0 args is still a type error

**The 1-arg and 2-arg-with-default forms must produce byte-identical output.** That is the
property; a test that only checks "1-arg works" would pass with a different default.

## STOP triggers

1. **If `:wat::edn::write` or `write-pretty` need to change — STOP.** They stay `Exact(1)`.
   Unchanged from WRITE-OPTS' STOP-1 and for the same reason: the Store sort-key path.
2. **If this needs a `Range`/`AtLeast` arity in the registry — STOP and report.** See above.
3. **If the floor reds on anything but the known journal arm — STOP**, capture whole, do NOT
   re-run.
4. **`.contains(` on a deterministic string trips `no_loose_string_assert`.** It has caught two
   stones in this excursus already. Use `assert_eq!` on the whole string from the start.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Floor here is **5113 with ONE known failure** (`probe_arc278_span_macros`, the journal
key-collision arm). **That red is expected and is not yours.**
