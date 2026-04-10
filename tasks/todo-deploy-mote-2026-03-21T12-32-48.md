# Todo: deploy mote macOS app bundle
- Status: BLOCKED (build failed due to I/O / disk-space issue)
- Parent: Work2026/mote/tasks/backlog.md
- Backlog item: #1 of 1
- Next: Free disk space on `/Volumes/Work2026` OR set `CARGO_TARGET_DIR` to local disk, then retry build
- W5H: 
  - Who: you (owner) + me (engineer agent)
  - What: build release and create macOS `.app` bundle for `mote`, install to `/Applications/Mote.app`
  - When: 2026-03-21T12:32:48+07:00 (HCMC local time)
  - Where: repository at `Work2026/mote` (mounted at `/Volumes/Work2026/mote`)
  - Why: distribute runnable macOS application for testers / users
  - How: run release build then execute `bundle-macos.sh` which assembles the `.app` bundle (script exists in repo). Verify artifact and compute checksum. Do NOT commit or push any git changes without explicit auth from user.
  
Issue summary (observed)
- While running `./bundle-macos.sh` the build failed with an I/O error writing to `target/release/deps/...` (error: Input/output error (os error 5)) and subsequent metadata read errors. Disk usage check shows the mounted workspace `/Volumes/Work2026` is effectively full (capacity reported 100%).
- Root cause: lack of free space / I/O on the remote/mounted volume prevented Cargo/rustc from writing build artifacts.

Immediate impact
- Build did not complete; `.app` bundle was not created/installed.
- No git operations were performed by me. The bundler invoked `cargo build --release` which failed halfway.

Recommended next steps (pick one)
1) Free space on `/Volumes/Work2026` (delete large unused files, move old `target/` or other caches off the volume), then retry:
   - Commands to run on host:
     - df -h /Volumes/Work2026
     - du -sh /Volumes/Work2026/* | sort -h
     - remove or archive large files (e.g. old backups, downloads)
   - After freeing >5-10 GiB, retry bundler:
     - cd /Volumes/Work2026/mote
     - ./bundle-macos.sh

2) Build on local disk (recommended if freeing remote space is slow):
   - Build using a local `CARGO_TARGET_DIR` to avoid writing to the mounted volume:
     - cd /Volumes/Work2026/mote
     - export CARGO_TARGET_DIR="$HOME/.cache/mote-target"
     - cargo build --release
     - If build succeeds, run the bundler but copy the binary from that target dir into the `.app` bundle, or modify `bundle-macos.sh` temporarily to `cp "${CARGO_TARGET_DIR}/release/${BINARY_NAME}" "${MACOS_DIR}/${BINARY_NAME}"`.
   - This avoids heavy writes to the mounted SMB volume.

3) Copy repository locally (fast temporary solution):
   - cp -a /Volumes/Work2026/mote ~/mote-local && cd ~/mote-local && ./bundle-macos.sh
   - After bundle creation, copy the `.app` back to /Applications or deliver by other means.

Notes
- The bundler performs ad-hoc codesign (`codesign -fs -`). If you need a real certificate or notarization later, we'll add that as a separate step.
- I will not perform deletions or git commits without your explicit instruction.

Logging evidence (captured before build was interrupted)
- Cargo failed with an I/O error writing to `target/release/deps/...` and later failed to open `.rmeta` metadata for `libc`. Disk usage for `/Volumes/Work2026` reported size near capacity (100%). Full logs are available in the execution output if you want them attached.

Please confirm which remediation you prefer:
- Free space on `/Volumes/Work2026` (I will pause; you clear space and tell me to retry), or
- I should switch to building on local disk via `CARGO_TARGET_DIR` and continue now, or
- You prefer I copy the repo locally and build there.


---
## Goal
Produce a signed (if necessary), packaged macOS app bundle at `/Applications/Mote.app` built from `Work2026/mote` and verify it launches. Provide build logs, artifact path, and SHA256 checksum.

## Preconditions / Assumptions
- Local machine is macOS and has required tooling: Rust toolchain (cargo), macOS codesign tools (codesign, productbuild) if script uses them.
- Repo is located at `/Volumes/Work2026/mote` and `bundle-macos.sh` is executable.
- No git commits or pushes will be made without explicit authorization.
- If codesigning or notarization is required, user will provide credentials or confirm manual steps.

## Checklist
- [x] 1) Verify repo exists and `bundle-macos.sh` is present and executable.
- [x] 2) Attempted `cargo build --release` via `./bundle-macos.sh` — build failed due to I/O / disk-space issue.
- [ ] 3) Remediation chosen and performed (free space on `/Volumes/Work2026` OR set `CARGO_TARGET_DIR` to local disk OR copy repo locally and build).
- [ ] 4) Re-run `./bundle-macos.sh` (or modified bundler that uses local target) to create the `.app` bundle.
- [ ] 5) Verify `/Applications/Mote.app` exists and list its size and contents.
- [ ] 6) Compute SHA256 of the bundle binary and report: `shasum -a 256 /Applications/Mote.app/Contents/MacOS/mote`.
- [ ] 7) Launch smoke test: attempt to open the app (headless: `open /Applications/Mote.app` and check `ps`), or run the binary directly to confirm exit code 0.
- [ ] 8) Collect artifacts: build log, `ls -lh` outputs, sha256, and a short run log (stdout/stderr).
- [ ] 9) Write review & lessons file if any steps failed or required changes.
- [ ] 10) Mark todo DONE and set Next to post-deploy smoke tests / notarization if needed.

## Commands (to run locally on your mac)
These are the exact commands I will run if you approve (run from host terminal):

1. Inspect repo and script:
   - cd /Volumes/Work2026/mote
   - ls -la
   - file ./bundle-macos.sh
   - ./bundle-macos.sh --help || true

2. Build release (explicit):
   - cd /Volumes/Work2026/mote
   - cargo build --release

3. Run bundler:
   - cd /Volumes/Work2026/mote
   - ./bundle-macos.sh

4. Verify install:
   - ls -l /Applications/Mote.app
   - ls -l /Applications/Mote.app/Contents/MacOS
   - shasum -a 256 /Applications/Mote.app/Contents/MacOS/mote

5. Smoke run:
   - open /Applications/Mote.app
   - # or run headless
   - /Applications/Mote.app/Contents/MacOS/mote --version
   - ps aux | grep -i mote | grep -v grep || true

Notes:
- If `bundle-macos.sh` performs a `cargo build --release` itself, step 2 can be skipped.
- If codesigning or notarization is required, the script may prompt for credentials or fail; in that case we will capture errors and ask you for next steps (provide certificates, app-specific password, or skip notarization).

## Verification criteria (exit conditions)
- `/Applications/Mote.app` exists and contains `Contents/MacOS/mote`.
- `shasum -a 256` for the binary is provided.
- The app can be launched (`open`) or the binary runs `--version` / exits 0 for a quick smoke test.
- Build and bundler logs captured and attached.

## Safety & Git policy
- I will not run any `git commit` or `git push` operations without your explicit approval and authentication. The bundler may touch `target/` only.
- If `bundle-macos.sh` attempts to modify repo files or create commits, it will be reported and paused for instruction.

## Estimated time
- Build + bundle: ~2–10 minutes depending on machine and whether a rebuild is required.
- Verification: 1–2 minutes.

## Review (to fill after execution)
- Build stdout/stderr: [attach]
- Bundle script stdout/stderr: [attach]
- Path to app: `/Applications/Mote.app`
- SHA256: [value]
- Smoke test result: [pass|fail]
- Follow-ups / Next steps: [e.g., notarize, automate CI, create release tarball]

## Lessons (if any failures occur)
- Create `tasks/lessons-deploy-mote-<timestamp>.md` summarizing root cause and prevention steps.

---
Created: 2026-03-21T12:32:48+07:00
Assigned to: me (engineer agent)