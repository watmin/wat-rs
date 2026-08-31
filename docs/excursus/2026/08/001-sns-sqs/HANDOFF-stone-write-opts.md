# HANDOFF → grok — excursus 001 stone WRITE-OPTS

⚠ **This work moved.** It was `docs/arc/2026/08/301-sns-sqs/` and is now
`docs/excursus/2026/08/001-sns-sqs/`. It is NOT an arc — arc 301 does not exist; the number was
minted unasked and retracted. Commit prefix is `EXCURSUS(001):`, never `STONE n(NNN):`. The six
`probe_arc301_*` tests are now `probe_ex001_*`. See `docs/excursus/README.md`.

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-write-opts.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-write-opts.md`

**The builder rejected three designs before this one.** Do not re-propose them:
a global config knob (a footgun — one setting and every `StoredRow` written afterwards loses
its range-scan ordering); a fixed default in `json.rs` (frozen from an assumption about a
consumer nobody asked); and a bare `digits` parameter (a timestamp concern on a general
serializer's signature — the wrong axis).

**What ships: a `WriteOpts` VALUE the caller passes**, on the `ProcessOpts` precedent already in
the tree at `wat/spawn.wat:77/122/130` — a struct, a zero-arg default constructor you never
touch, and named single-field variants. This excursus's own SNS demo uses both halves of that
pattern already.

⛔ **`:wat::edn::write` (the 1-arg EDN verb) does not change.** 424 call sites, and it is the
`Store` sort-key path — its width is a correctness invariant, not a preference. If opts cannot
be added to the JSON verbs without touching it, that is a finding, not a licence.

Verify in the FOREGROUND; read the Summary line, never a piped exit code. Floor here is **5103
with ONE known failure** (`probe_arc278_span_macros`, the journal key-collision arm). **That red
is expected and is not yours.** Two failures means you added one. On a NEW red: do NOT re-run,
capture the arm whole, name the exact assertion.
