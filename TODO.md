# StreamFlow-Tauri - TODO & Issues

## 🔴 Critical (Blocking)

- [x] **1. Fix CI/CD environment variable passing**
  - File: `.github/workflows/ci.yml`
  - Issue: `RELEASE_VERSION` not passed between jobs
  - Fix: Use job outputs instead of env vars
  - ✅ DONE

- [x] **2. Remove Cargo.lock from .gitignore**
  - File: `.gitignore`
  - Issue: Causes non-reproducible builds and CI cache failures
  - Fix: Remove `Cargo.lock` from gitignore
  - ✅ DONE

- [x] **3. Fix duplicate invoke declaration**
  - File: `src/script.js`
  - Issue: SyntaxError - `const { invoke }` declared twice
  - Fix: Remove duplicate
  - ✅ DONE

- [x] **4. Fix missing events.js import**
  - File: `src/script.js`
  - Issue: `import('./events.js')` references non-existent file
  - Fix: Replace with inline VLC check
  - ✅ DONE

- [x] **5. Add #[tauri::command] to get_platform**
  - File: `src-tauri/src/util/helpers.rs`
  - Issue: Function registered as command but missing attribute
  - Fix: Add `#[tauri::command]` attribute
  - ✅ DONE

- [x] **6. Initialize Tauri plugins**
  - File: `src-tauri/src/main.rs`
  - Issue: Plugins declared in Cargo.toml but never initialized
  - Fix: Add `.plugin()` calls for shell, process, http
  - ✅ DONE

## 🟡 Major (Should Fix)

- [x] **7. Sync version numbers**
  - Files: All project files
  - Fix: Updated to 1.1.0
  - ✅ DONE

- [x] **8. Add working-directory to CI cargo commands**
  - File: `.github/workflows/ci.yml`
  - Fix: Added working-directory to all jobs
  - ✅ DONE

- [x] **9. Add versioned release trigger**
  - File: `.github/workflows/ci.yml`
  - Fix: Added versioned-release job with tag trigger
  - ✅ DONE

- [x] **10. Add frontend linting to CI**
  - File: `.github/workflows/ci.yml`
  - Fix: Added auto-fix job with Prettier and cargo fmt
  - ✅ DONE

- [x] **11. Fix branch naming (master → main)**
  - All workflow references updated to use `main`
  - ✅ DONE

- [x] **12. Fix OnceLock caching issue**
  - File: `src-tauri/src/streamlink.rs`
  - Issue: VLC availability cache could never update
  - Fix: Changed to Mutex for updateable caching
  - ✅ DONE

- [x] **13. Improve mutex error handling**
  - File: `src-tauri/src/functionality/streaming.rs`
  - Issue: `.unwrap()` on mutex locks can panic
  - Fix: Use `.map_err()` for graceful handling
  - ✅ DONE

## 🟢 Minor (Nice to Have / Tech Debt)

- [x] **14. Consolidate duplicate types**
  - Files: `config.rs` and `settings.rs` both defined `SavedStream`, `StreamStatus`
  - Fix: Created shared `types.rs` module, updated both files to import from it
  - ✅ DONE

---

## Progress Summary

| Priority | Total | Done |
|----------|-------|------|
| 🔴 Critical | 6 | 6 |
| 🟡 Major | 7 | 7 |
| 🟢 Minor | 1 | 1 |

## ✅ All tasks complete!

## CI/CD Features

- **Auto-fix formatting**: Automatically formats code on push and commits fixes
- **Code quality**: Runs rustfmt and clippy checks
- **Build**: Compiles and tests on Windows x64 MSVC
- **Rolling release**: Creates artifacts on every push to main
- **Versioned release**: Creates GitHub releases when pushing `v*.*.*` tags
