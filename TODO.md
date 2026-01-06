# StreamFlow-Tauri - TODO & Issues

## 🔴 Critical (Blocking)

- [x] **1. Fix CI/CD environment variable passing**
  - File: `.github/workflows/ci.yml`
  - Issue: `RELEASE_VERSION` not passed between jobs (lines 131, 132, 167, 176)
  - Fix: Use job outputs instead of env vars
  - ✅ DONE: Added job outputs and proper variable passing

- [x] **2. Remove Cargo.lock from .gitignore**
  - File: `.gitignore` (line 3)
  - Issue: Causes non-reproducible builds and CI cache failures
  - Fix: Remove `Cargo.lock` from gitignore, generate and commit it
  - ✅ DONE: Updated .gitignore (run `cargo generate-lockfile` locally)

- [ ] **3. Fix duplicate invoke declaration**
  - File: `src/script.js` (lines 3 and 199)
  - Issue: SyntaxError - `const { invoke }` declared twice
  - Fix: Remove line 199

- [ ] **4. Fix missing events.js import**
  - File: `src/script.js` (line 842)
  - Issue: `import('./events.js')` references non-existent file
  - Fix: Replace with inline `isVlcRunning()` check

- [ ] **5. Add #[tauri::command] to get_platform**
  - File: `src-tauri/src/util/helpers.rs` (line 5)
  - Issue: Function registered as command but missing attribute
  - Fix: Add `#[tauri::command]` attribute

- [ ] **6. Initialize Tauri plugins**
  - File: `src-tauri/src/main.rs` (line 41)
  - Issue: Plugins declared in Cargo.toml but never initialized
  - Fix: Add `.plugin()` calls for shell, process, http

## 🟡 Major (Should Fix)

- [ ] **7. Sync version numbers**
  - Files: `src-tauri/Cargo.toml` (1.0.0) vs `package.json` (1.0.1)
  - Fix: Update Cargo.toml to 1.0.1

- [x] **8. Add working-directory to CI cargo commands**
  - File: `.github/workflows/ci.yml`
  - Issue: Cargo commands run from root, not src-tauri/
  - Fix: Add `working-directory: src-tauri` or `--manifest-path`
  - ✅ DONE: Added working-directory to all jobs

- [x] **9. Add versioned release trigger**
  - File: `.github/workflows/ci.yml`
  - Issue: No `v*.*.*` tag trigger for semantic releases
  - Fix: Add tags trigger to workflow
  - ✅ DONE: Added versioned-release job with tag trigger

- [x] **10. Add frontend linting to CI**
  - File: `.github/workflows/ci.yml`
  - Issue: JavaScript errors not caught in CI
  - Fix: Add Prettier check step
  - ✅ DONE: Added frontend-lint job

## 🟢 Minor (Nice to Have)

- [ ] **11. Consolidate duplicate types**
  - Files: `config.rs` and `settings.rs` both define `SavedStream`, `StreamStatus`
  - Fix: Create shared `types.rs` module

- [ ] **12. Remove unused dependencies**
  - File: `src-tauri/Cargo.toml`
  - Issue: `base64`, `gumdrop`, `winapi` never used
  - Fix: Remove from Cargo.toml

- [ ] **13. Fix OnceLock caching issue**
  - File: `src-tauri/src/streamlink.rs`
  - Issue: VLC availability cache can never update
  - Fix: Use `Mutex` instead of `OnceLock`

- [ ] **14. Improve mutex error handling**
  - File: `src-tauri/src/functionality/streaming.rs`
  - Issue: `.unwrap()` on mutex locks can panic
  - Fix: Use `.map_err()` for graceful handling

---

## Progress

| Priority | Total | Done |
|----------|-------|------|
| 🔴 Critical | 6 | 0 |
| 🟡 Major | 4 | 0 |
| 🟢 Minor | 4 | 0 |
