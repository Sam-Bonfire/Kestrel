# Contributing to Kestrel

Welcome to Kestrel! We're thrilled that you want to contribute to our self-hosted email and calendar suite. This guide will help you get set up and understand our workflow so you can start contributing quickly and effectively.

Kestrel uses a Rust backend, Svelte frontends (using modern `$state()` runes), and Tauri for native desktop and mobile applications.

## Development Environment Setup

### Prerequisites
Before you begin, ensure you have the following installed:
- [mise](https://mise.jdx.dev/) - For managing tool versions (Node, Rust, etc.)
- [Docker](https://www.docker.com/) - For running necessary services (like the database)
- [Jujutsu (jj)](https://martinvonz.github.io/jj/latest/) or [Git](https://git-scm.com/) - For version control

### Step-by-Step Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/kestrel.git
   cd kestrel
   ```
   *(If you're using jj, use your standard `jj git clone` flow)*

2. **Install tools and dependencies:**
   We strictly use `pnpm` for package management and `mise` for tasks. You can set everything up in one command:
   ```bash
   mise run init
   ```
   *This installs the toolchain (Rust, Node), installs pnpm dependencies, builds the shared library, and fetches Cargo dependencies.*

### Project Structure
Kestrel is a monorepo containing both the backend and multiple frontend applications.

| Directory | Description |
|-----------|-------------|
| `backend/` | Rust-based backend API and WASM plugin runtime |
| `frontend-mail/` | Svelte & Tauri frontend for the Mail application |
| `frontend-calendar/` | Svelte & Tauri frontend for the Calendar application |
| `packages/shared/` | Shared TypeScript library (must be built before running frontends) |

## Development Workflow

### 1. Build the Shared Library
Before running the frontends, you must build the shared library:
```bash
pnpm --filter @kestrel/shared build
```

### 2. Running the Backend
The Rust backend requires a few environment variables. You can typically set these in a `.env` file in the `backend/` directory.

```bash
cd backend
DATABASE_URL=postgres://user:pass@localhost/kestrel PORT=3000 HOST=127.0.0.1 cargo run
```

### 3. Running the Frontends
You can run the web UI only, or the full Tauri native application.

**Mail App (Tauri):** runs on port 1420
```bash
cd frontend-mail
pnpm tauri dev
```

**Calendar App (Tauri):** runs on port 1421
```bash
cd frontend-calendar
pnpm tauri dev
```

**Web UI only (no Tauri):**
```bash
pnpm dev
```

### 4. Running Tests
- **Backend:** `cargo test --manifest-path backend/Cargo.toml`
- **Frontend Build Check:** `pnpm --filter frontend-mail build`

## Branch Strategy

- **Feature branches** branch from and create PRs against `dev`.
- **`dev`** is the main integration branch. All CI checks must pass here.
- **`main`** is the release branch. We use a manual PR from `dev` to `main` which triggers auto-tagging and releasing.

**Branch Naming Convention:**
- `feature/description`
- `fix/description`
- `docs/description`

## Code Style

### Rust
We follow standard Rust idioms enforced by cargo tools. Before committing, ensure you run:
```bash
cargo fmt
cargo clippy -- -D warnings
```

### Frontend
- Follow existing Svelte and Tailwind CSS patterns in the codebase.
- We use modern Svelte 5 syntax. Use `$state()` runes instead of legacy reactive declarations (`$: ...`).

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/).
Allowed prefixes: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`

**Examples:**
- `feat(calendar): add week view drag-to-create`
- `fix(mail): correct thread sorting order`
- `docs: update README setup instructions`

## Adding a Provider Plugin

Kestrel uses a WASM-based plugin architecture for providers.

1. Plugins live in the `backend/src/plugins/` directory.
2. Your plugin must implement the `ProviderPlugin` trait.
3. The real plugins are executed using the Wasmtime runtime (`backend/src/plugins/wasm_runtime.rs`).

<details>
<summary>View mock implementation reference</summary>

You can find a reference mock implementation in `backend/src/plugins/mock.rs`. Use this as a guide for how to structure your plugin's data and trait implementation before compiling it to WASM.
</details>

## Pull Request Guidelines

1. **Target the `dev` branch:** All standard PRs should go to `dev`.
2. **Describe your changes:** Provide a clear description of what the PR does and why.
3. **Pass CI:** Ensure all backend checks, frontend builds, and mobile builds pass on your PR.
4. **Keep it focused:** Try to limit each PR to a single feature or bug fix to make reviewing easier.

## Versioning

Kestrel's version is tracked across multiple files. **If you are preparing a release**, these 5 files must be updated together and kept in sync:

- `frontend-mail/src-tauri/tauri.conf.json`
- `frontend-calendar/src-tauri/tauri.conf.json`
- `frontend-mail/src-tauri/Cargo.toml`
- `frontend-calendar/src-tauri/Cargo.toml`
- `backend/Cargo.toml`

Our release pipeline validates this consistency. Bump all 5 files before merging a release PR to `main`.
