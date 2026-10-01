# BRIEF — shape strike T2: a malformed request stays structured, end to end

Excursus 003. This continues strike T (`BRIEF-shape-strike-T-typed-decode-checks-every-field.md`).
T's implementation sits **uncommitted** in the working tree on top of `f74359851`, because its floor is
red on exactly 4 tests, and this strike makes them green. **Do not discard that tree.** You land T and
T2 together, or T2 on top of T in its own commit.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## What T built (uncommitted; verify with `git diff --stat`)

- `value_conforms` (`src/edn/render.rs`): one checker of a value against a declared type.
- Every reconstructor checks every field, and refuses undeclared keys:
  - struct, record, holon-record, enum-variant;
  - `coerce_struct_path` refuses undeclared keys.
- New refusals: `FieldTypeMismatch`, `UnknownField`, `UndeclaredFieldType`.
- `:wat::core::Error` is registered as a builtin surface. `register_builtin` gained its Surface arm,
  the Record subtype edge.
- `rewrap_tuple_field`: EDN has no tuple literal, so a tuple field is rewrapped by its declared type,
  as `Option` already is.
- `pv()` in `src/rete/export.rs` now builds a `PersistentVector`, and `tests/rete/datamancer.rete.edn`
  is regenerated.
- All of T's gates (GT1–GT3, the ruling-4 arms, the tuple rewrap) are mutation-proven.

## The defect T found, which this strike cures

On the **process tier**, a service request arrives as wire bytes. `recv'` (`src/kernel/message.rs`,
several sites) decodes it with `decode_trusted_wire` **before** the dispatch arm's
`:wat::edn::validate` runs.

A decode failure is **flattened to prose** at that point:
`message_only_failure(format!("... decode failed: {}", e))` (e.g. `src/kernel/message.rs:~1905`).
It becomes `ServiceEvent::Malformed{idx, cause}` (`wat/spawn.wat:~195`), and `wat/service.wat:~1960`
replies `Reply::Failed[cause]`.

With T's strict decode, a correctly tagged but wrong-typed request is now refused at `recv'`. So:
- arc 278's **`:RequestMalformed`** reply (structured `path`/`expected`/`got`, `wat/service.wat`)
  **no longer fires** for this attack;
- the client gets a sentence instead.

There is no denial of service: the server keeps the client and keeps serving (T traced this). But the
contract the builder ruled is broken, and the structure is stringified, which is the very defect this
excursus exists to remove.

On the **thread tier** the `Value` crosses verbatim and is never decoded, so there is no wire to
attack. T measured that `:wat::edn::validate`'s `Invalid` arm is now very likely dead for its stated
threat model. It stays as defence in depth: **do not delete it.**

The 4 red tests are `deftest_wat_tests_service_{request_malformed,parametric_messages_round_trip}_on_{thread,process}`
(`wat-tests/service-request-malformed.wat:~111`, `wat-tests/service-parametric-messages.wat:~181`).
They build a poisoned request **value** with `:wat::edn::read`, which strict decode now refuses by
design.

## Target

1. **A decode failure in `recv'` stays structure.** At every `decode_trusted_wire` failure site in
   `src/kernel/message.rs`, the `Failure.error` is a **declared record of the read error**, not
   `message_only_failure(format!(…))`.
   - Declare the `EdnReadErrorKind` family in wat if it is not declared yet (the sweep's pattern:
     records mirror the wire, Rust derived from the `.wat`). At minimum, cover the kinds that can
     arise here: `FieldTypeMismatch`, `UnknownField`, `UndeclaredFieldType`, `UnknownTag`,
     `UnknownStructField` and the parse failures.
   - Measure which kinds can arise, and name any you leave undeclared, with the reason.
2. **Request-shape refusals reply `:RequestMalformed`.** When the refusal is about the request's
   **shape** (a field type, an unknown field, an undeclared type, a missing declared field) on a
   request-position value, the serve loop replies the op's existing `:RequestMalformed`. It carries
   `path` (the dotted `field` split into `Vector<String>` segments, in the contract's existing form),
   `expected` and `got`, all taken from the structured refusal.
   - Any other decode failure (an unparseable frame, an unknown tag) keeps the generic
     `ServiceEvent::Malformed` → `Reply::Failed`, now with a structured cause.
   - Read the arc-278 request-sanitization design first, to find which op the reply belongs to and
     how `:RequestMalformed` is already built (`wat/service.wat` around the `shape-guarded` / `:wat::edn::validate`
     arm). **Reuse that construction; do not build a second one.**
3. **The two tests send the real attack.** Convert both tests to send a raw, correctly tagged,
   wrong-typed request **frame** on the process tier, and assert `:RequestMalformed` with the exact
   `path`/`expected`/`got`.
   - On the thread tier there is no wire. Measure whether the thread-tier variant of each test still
     has a meaningful claim, now that no wat path can build a wrong-typed request value:
     - if it does, keep it with an honest assertion;
     - if it does not, retire it, and say so in the test file, citing T's measurement.
   - T said a raw-socket test is hard: the address is not exposed, there is a peer-credential gate,
     and the Op envelope must be exact. **Find the honest door.**
     - It might be a test-only helper that writes raw bytes on an established client connection, or
       the existing typed client with a primitive that emits a raw frame.
     - If no honest door exists without new kernel surface, STOP and report what is missing.
     - Do **not** fake it with an in-process value: that is the exact thing strict decode now
       forbids.

## Gates (each mutation-proven in RELEASE)

- **GT2a, the decode cause is structured.** A malformed process-tier frame's `ServiceEvent::Malformed`
  cause (or `:RequestMalformed`) carries the read-error record class, not prose.
  - Mutation: restore `message_only_failure(format!(…))` at one site. RED.
- **GT2b, the contract fires.** The two converted tests assert `:RequestMalformed` with the exact
  `path`/`expected`/`got`.
  - Mutation: send shape refusals to the generic `Reply::Failed`. RED.
- **GT2c, a non-shape failure stays generic.** An unparseable frame still gets `Reply::Failed`,
  structured, and the server keeps serving.
- All of T's gates stay green.

## Then land it

- Run the floor: `scripts/floor.sh` in the foreground, 0 failed, 0 timed out.
- Run `git diff --stat -- '*.edn'`. Only `tests/rete/datamancer.rete.edn` (T's `pv()` fix) and any
  goldens this strike legitimately changes may move. Read every one.
- Commit T, and T2 if it is separable, with the prefix `EXCURSUS(003):`, and push. The T commit
  message must carry T's census:
  - the overruled `Ok(())` fallback;
  - the Surface subtype-edge bug;
  - the tuple rewrap;
  - `pv()`, and the `PersistentVector` tag overhead that made `export_edn_is_smaller_than_session`
    false (T renamed that test to what is true);
  - `:wat::edn::validate` now likely dead for its stated threat model, kept.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- The floor: on any red, surface it before any re-run, verbatim, with the arm named. On a timeout,
  surface it with its history; do not widen `.config/nextest.toml`.
- Stage by name BEFORE the floor. Never `git add -A`. No `cargo fmt`.
- `.wat` multi-site edits go through the wat-fix codemod. Two test files plus one stdlib arm are small
  enough to edit by hand.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary and report where.** Never commit a red floor.

## Report

- which read-error kinds are declared;
- the decode sites changed;
- the raw-frame door you found, or what is missing;
- the thread-tier decision for each test;
- each gate's mutation RED;
- any golden that moved;
- the floor `Summary` line, verbatim;
- the SHA(s).
