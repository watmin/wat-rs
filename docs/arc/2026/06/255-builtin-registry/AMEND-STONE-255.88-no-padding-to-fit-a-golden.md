# AMEND — STONE 255.88: no padding to fit a golden

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `05e0c331b`.
Commit locally on `main`; **do not push**.

The stone is accepted in substance: the probe prints `0 7 42 1 99`; `u/a/b` and the pathological name are callable (the
`defn` name block split first-and-last segment; now `canonical-identity`); binders are `$bound` by position
(`Identifier::into_bound`); the doc-link judge is a bin and the ignore ledger is back to 18; `flat` measured (504,923
`as_str` calls on one probe) and kept.

**One commit is withdrawn: `d916e49c4`** adds a bare `;;` line to `wat/core.wat` so five span goldens keep their line
numbers. That is the program's text bent to fit a golden, the same shape as raising a limit to fit a gate. **Revert it**,
and re-capture the five goldens exactly as 255.87 amendment 6 did: show, as data, that each differs from its pre-image
only by the `wat/core.wat` line number (message, reason, file, columns unchanged), then re-capture.

**Record the fragility in the SCORE (do not cure it here):** these goldens pin line numbers **inside the stdlib**, so any
`wat/core.wat` edit above them churns tests unrelated to it (twice today: 255.87 +3, 255.88 −1). Name the five tests and
what each golden's span is there to prove, so a later stone can decide whether they should assert the span's **file and
form** rather than a line number in another file.

Then the floor, clippy, census `--diff`; `git status` clean before the floor. A STOP means STOP. Append to the SCORE,
commit, **do not push**.
