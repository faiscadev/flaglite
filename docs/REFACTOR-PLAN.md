# FlagLite Monorepo Refactor Plan

**Goal:** Transform FlagLite into a proper Cargo workspaces monorepo following the patterns established in `faiscadev/zopp`.

**Reference:** https://github.com/faiscadev/zopp

---

## Current State

```
flaglite/
├── api/                    # API server binary
├── cli/                    # CLI binary  
├── shared/                 # Shared library (client, types)
├── sdks/                   # Language SDKs (JS, Python, Go)
├── dashboard/              # React dashboard (separate)
├── website/                # Astro marketing site
├── helm/                   # Helm chart
├── e2e/                    # Shell-based e2e tests
└── .github/workflows/      # Mixed workflows
```

**Issues:**
- Flat structure without clear separation
- E2E tests are shell scripts, not Rust
- No lint workflow (fmt, clippy)
- No coverage tracking
- No xtask for dev tooling
- Shared crate mixes client, types, and error handling
- Storage implementations are in `api/`, not separate crates

---

## Target State

```
flaglite/
├── apps/
│   ├── flaglite-api/       # API server binary
│   ├── flaglite-cli/       # CLI binary
│   ├── flaglite-web/       # Future: dashboard as Rust (Leptos?)
│   └── e2e-tests/          # Rust e2e test crate
├── crates/
│   ├── flaglite-client/    # HTTP client library
│   ├── flaglite-config/    # Configuration management
│   ├── flaglite-core/      # Core types, traits, errors
│   ├── flaglite-storage/   # Storage trait definitions
│   ├── flaglite-store-sqlite/   # SQLite implementation
│   └── flaglite-store-postgres/ # PostgreSQL implementation
├── sdks/                   # Non-Rust SDKs (JS, Python, Go)
├── website/                # Marketing site (Astro)
├── charts/                 # Helm charts
├── docker/                 # Dockerfiles
├── scripts/                # Helper scripts
├── xtask/                  # Development tasks (cargo xtask)
├── .github/
│   └── workflows/
│       ├── lint.yaml       # fmt + clippy
│       ├── test.yaml       # Unit tests + coverage
│       ├── e2e.yaml        # Integration tests
│       ├── build.yaml      # Build check
│       ├── docker.yaml     # Container builds
│       └── release.yaml    # CLI releases
├── Cargo.toml              # Workspace root
├── Cargo.lock
├── rust-toolchain.toml     # Pin Rust version
└── .cargo/config.toml      # Cargo configuration
```

---

## Phase 1: Structure Migration (Foundation)

**Goal:** Reorganize existing code without breaking functionality.

### Tasks

1. **Create directory structure**
   ```bash
   mkdir -p apps/flaglite-api apps/flaglite-cli apps/e2e-tests
   mkdir -p crates/flaglite-core crates/flaglite-client crates/flaglite-config
   mkdir -p crates/flaglite-storage crates/flaglite-store-sqlite crates/flaglite-store-postgres
   mkdir -p docker scripts xtask
   mv helm charts
   ```

2. **Move api/ → apps/flaglite-api/**
   - Update Cargo.toml package name
   - Update internal imports

3. **Move cli/ → apps/flaglite-cli/**
   - Update Cargo.toml package name
   - Update internal imports

4. **Split shared/ into multiple crates:**
   - `crates/flaglite-core/` - Types, errors, traits
   - `crates/flaglite-client/` - HTTP client
   - `crates/flaglite-config/` - CLI configuration

5. **Extract storage from api/:**
   - `crates/flaglite-storage/` - Trait definitions
   - `crates/flaglite-store-sqlite/` - SQLite impl
   - `crates/flaglite-store-postgres/` - Postgres impl

6. **Update root Cargo.toml:**
   ```toml
   [workspace]
   members = [
       "apps/flaglite-api",
       "apps/flaglite-cli", 
       "apps/e2e-tests",
       "crates/flaglite-core",
       "crates/flaglite-client",
       "crates/flaglite-config",
       "crates/flaglite-storage",
       "crates/flaglite-store-sqlite",
       "crates/flaglite-store-postgres",
       "xtask",
   ]
   resolver = "2"
   
   [workspace.package]
   version = "0.1.0"
   edition = "2021"
   rust-version = "1.80"
   license = "AGPL-3.0"
   repository = "https://github.com/faiscadev/flaglite"
   
   [workspace.dependencies]
   # All shared deps here
   ```

7. **Create rust-toolchain.toml:**
   ```toml
   [toolchain]
   channel = "stable"
   ```

8. **Create .cargo/config.toml:**
   ```toml
   [alias]
   xtask = "run --package xtask --"
   ```

### Verification
- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] All existing functionality works

---

## Phase 2: CI/CD Modernization

**Goal:** Adopt zopp's CI patterns for quality gates.

### New Workflows

#### `.github/workflows/lint.yaml`
```yaml
name: Lint

on: push

concurrency:
  group: lint-${{ github.ref }}
  cancel-in-progress: true

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: rustfmt
        run: cargo fmt --all -- --check
      - name: clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
```

#### `.github/workflows/test.yaml`
- Unit tests with coverage (cargo-llvm-cov)
- Coverage reporting to PR comments
- Coverage badge on GitHub Pages
- Regression detection

#### `.github/workflows/e2e.yaml`  
- Rust-based e2e tests in `apps/e2e-tests/`
- Postgres service container
- Runs actual CLI + API integration

#### `.github/workflows/build.yaml`
- Build check on all targets
- Cache optimization

### Tasks

1. Create `lint.yaml` workflow
2. Create `test.yaml` with coverage (from zopp)
3. Create Rust e2e test crate
4. Create `e2e.yaml` workflow
5. Update `build.yaml` for workspace
6. Add concurrency groups to all workflows
7. Add `Swatinem/rust-cache@v2` to all workflows

### Verification
- [ ] `cargo fmt --check` passes in CI
- [ ] `cargo clippy` passes in CI
- [ ] Coverage reports on PRs
- [ ] E2E tests run in CI

---

## Phase 3: Developer Experience

**Goal:** Add xtask and improve local development.

### xtask Commands

```rust
enum Commands {
    /// Set up development database
    SetupDb { backend: String },
    
    /// Prepare sqlx offline metadata
    SqlxPrepare { backend: String },
    
    /// Run all checks (fmt, clippy, test)
    Check,
    
    /// Generate OpenAPI spec from code
    OpenApi,
    
    /// Run local development server
    Dev,
}
```

### Tasks

1. Create `xtask/` crate
2. Implement `setup-db` for SQLite/Postgres
3. Implement `sqlx-prepare` for offline builds
4. Implement `check` (fmt + clippy + test)
5. Add `scripts/unit-coverage.sh` from zopp
6. Update CONTRIBUTING.md with new workflow

### Verification
- [ ] `cargo xtask setup-db sqlite` works
- [ ] `cargo xtask check` passes
- [ ] Local dev workflow documented

---

## Phase 4: E2E Tests Migration

**Goal:** Replace shell e2e tests with Rust tests.

### New Test Structure

```
apps/e2e-tests/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    ├── auth_test.rs
    ├── projects_test.rs
    ├── flags_test.rs
    └── cli_test.rs
```

### Test Categories

1. **Auth Tests**
   - Signup with auto-generated username
   - Signup with custom username
   - Login with correct/wrong password
   - API key authentication
   - JWT token authentication

2. **Projects Tests**
   - List projects (default created on signup)
   - Create new project
   - Project auto-creates environments

3. **Flags Tests**  
   - Create flag
   - List flags
   - Get flag by key
   - Toggle flag
   - Delete flag

4. **CLI Tests**
   - `flaglite signup`
   - `flaglite login`
   - `flaglite projects list`
   - `flaglite flags create`
   - `flaglite flags list`

### Tasks

1. Create `apps/e2e-tests/` crate
2. Add test harness (spawn server, cleanup)
3. Port auth tests from shell
4. Port projects tests
5. Port flags tests
6. Add CLI subprocess tests
7. Remove old `e2e/test-api.sh`

### Verification
- [ ] `cargo test --package e2e-tests` passes
- [ ] CI runs e2e tests
- [ ] Shell script removed

---

## Phase 5: Polish

**Goal:** Final cleanup and documentation.

### Tasks

1. **Clean up repository root**
   - Remove orphan files (flaglite.db, test.db)
   - Update .gitignore
   - Add .dockerignore

2. **Update documentation**
   - README.md with new structure
   - CONTRIBUTING.md with dev workflow
   - Architecture diagram

3. **Standardize Dockerfiles**
   - Move to `docker/` directory
   - api.Dockerfile
   - cli.Dockerfile

4. **Update SDK CI**
   - SDKs should depend on e2e passing
   - Version bumping automation

### Verification
- [ ] Clean repo root
- [ ] Docs updated
- [ ] All CI green

---

## Migration Order

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5
   ↓         ↓         ↓         ↓         ↓
 Structure  CI/CD    DevEx     E2E      Polish
 (2 days)  (1 day)  (1 day)  (2 days)  (1 day)
```

**Total estimated time:** 7 days

---

## Risk Mitigation

1. **Breaking changes:** Keep old paths working during migration with re-exports
2. **CI failures:** Migrate workflows incrementally, keep old ones until new pass
3. **Import chaos:** Use workspace dependencies to centralize versions

---

## Success Criteria

- [ ] `cargo build --workspace` completes in <2 minutes (cached)
- [ ] `cargo clippy --workspace` has 0 warnings
- [ ] `cargo fmt --check` passes
- [ ] Test coverage >70%
- [ ] E2E tests cover all critical paths
- [ ] Single `cargo xtask check` runs all quality gates
- [ ] New developer can set up in <5 minutes

---

## Notes

- Keep `sdks/`, `website/`, `dashboard/` outside Cargo workspace (non-Rust)
- Consider moving dashboard to Leptos later (Phase N+1)
- Helm charts stay in `charts/` (standard convention)
