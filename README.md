# Appian Deployment CLI (Binary + Source)

A Windows-friendly command-line tool to automate Appian deployments via the Appian Deployment REST API v2.
This repository intentionally includes only:

- `src/`  the Rust source code
- `appian-deployment-cli.exe`  the prebuilt Windows x86_64 binary

Everything else (docs, tests, manifests) is omitted to keep distribution minimal.
Use the binary directly, or adapt the source in your own workspace.

## Configuration
- Config file: place `appian-config.toml` next to the binary.
- Required keys:
  - `base_url`  your Appian site root, e.g. `https://mysite.appiancloud.com`
  - `api_key`  API key with access to the Deployment API v2
  - `timeout_seconds`  request timeout (default `300`)

Example `appian-config.toml`:
```toml
base_url = "https://mysite.appiancloud.com"
api_key = "your-api-key-here"
timeout_seconds = 300
```

- Environment variables (override file when set):
  - `APPIAN_BASE_URL`, `APPIAN_API_KEY`, `APPIAN_TIMEOUT_SECONDS`
- CLI global overrides (highest precedence):
  - `--base-url`, `--api-key`, `--config-file`, `--format <text|json>`, `--verbose`, `--quiet`

Precedence: CLI overrides > environment variables > config file.

## Building on macOS/Linux
- Install Rust via rustup:
  - macOS/Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Verify: `rustc -V` and `cargo -V`
- Build the binary:
  - `cargo build --release`
- Binary path:
  - macOS/Linux: `./target/release/appian-deployment-cli`
  - Windows: `target\release\appian-deployment-cli.exe`
- TLS backend:
  - Default uses Rustls (`--features rustls-tls` implied).
  - To use native platform TLS: `cargo build --release --features native-tls`
- Run examples below using your platform’s binary name. Where you see `.exe` for Windows, use `./appian-deployment-cli` on macOS/Linux.

## Command Reference
Global options apply to all commands: `--config-file`, `--base-url`, `--api-key`, `--format <text|json>`, `--verbose`, `--quiet`.

### get-packages
List packages for one or more applications.
- Flags:
  - `--app-uuid <UUID>` (repeatable)
- Example:
```powershell
./appian-deployment-cli.exe get-packages --app-uuid 11111111-1111-1111-1111-111111111111 --app-uuid 22222222-2222-2222-2222-222222222222 --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json
```

```bash
./appian-deployment-cli get-packages --app-uuid 11111111-1111-1111-1111-111111111111 --app-uuid 22222222-2222-2222-2222-222222222222 --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json
```

### export
Export applications or a single package to an artifact zip.
- Flags:
  - `--uuids <UUID[,UUID,...]>` (repeatable or comma-separated)
  - `--export-type <package|application>` (default `package`)
  - `--name <STRING>` (optional)
  - `--description <STRING>` (optional)
  - `--dry-run` (validation only; does not call server)
- Rules:
  - When `export-type=package`, exactly one UUID is required.
  - When `export-type=application`, one or more UUIDs are allowed.
- Examples:
```powershell
# Package export (exactly one UUID)
./appian-deployment-cli.exe export --uuids 11111111-1111-1111-1111-111111111111 --export-type package --name "Sample Export" --description "Testing" --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json

# Application export (multiple UUIDs)
./appian-deployment-cli.exe export --uuids 11111111-1111-1111-1111-111111111111,22222222-2222-2222-2222-222222222222 --export-type application --name "My App Export" --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json

# Validate only
./appian-deployment-cli.exe export --uuids 11111111-1111-1111-1111-111111111111 --export-type package --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --dry-run
```

```bash
# Package export (exactly one UUID)
./appian-deployment-cli export --uuids 11111111-1111-1111-1111-111111111111 --export-type package --name "Sample Export" --description "Testing" --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json

# Application export (multiple UUIDs)
./appian-deployment-cli export --uuids 11111111-1111-1111-1111-111111111111,22222222-2222-2222-2222-222222222222 --export-type application --name "My App Export" --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json

# Validate only
./appian-deployment-cli export --uuids 11111111-1111-1111-1111-111111111111 --export-type package --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --dry-run
```

### inspect
Submit a package for inspection (pre-deployment checks).
- Flags:
  - `--package-zip-name <PATH>` (required)
  - `--customization-file <PATH>` (.properties, optional)
  - `--admin-console-file <PATH>` (.zip, optional)
- Example:
```powershell
./appian-deployment-cli.exe inspect --package-zip-name .\artifacts\my_package.zip --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json
```

```bash
./appian-deployment-cli inspect --package-zip-name ./artifacts/my_package.zip --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json
```

### get-inspection
Retrieve inspection results by inspection UUID.
- Flags:
  - `--uuid <UUID>` (required)
- Example:
```powershell
./appian-deployment-cli.exe get-inspection --uuid 00000000-0000-0000-0000-000000000000 --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json
```

```bash
./appian-deployment-cli get-inspection --uuid 00000000-0000-0000-0000-000000000000 --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json
```

### deploy
Deploy an exported package to a target environment.
- Flags:
  - `--package-zip-name <PATH>` (required)
  - `--name <STRING>` (required)
  - `--description <STRING>` (optional)
  - `--dry-run` (plan-only; validates inputs)
  - `--rollback-on-failure` (default `true`)
  - `--customization-file <PATH>` (.properties, optional)
  - `--admin-console-file <PATH>` (.zip, optional)
  - `--plugins-file <PATH>` (.zip, optional)
  - `--data-source <NAME|UUID>` (optional)
  - `--database-scripts <PATH[,PATH,...]>` (comma-separated; executed in order)
- Examples:
```powershell
# Dry run to validate inputs
./appian-deployment-cli.exe deploy --package-zip-name .\artifacts\my_package.zip --name "My Deploy" --description "Testing" --rollback-on-failure --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --dry-run

# Real deployment with optional files
./appian-deployment-cli.exe deploy --package-zip-name .\artifacts\my_package.zip --name "My Deploy" --customization-file .\configs\import.properties --admin-console-file .\configs\admin.zip --plugins-file .\plugins\plugins.zip --database-scripts .\db\scripts\01_schema.sql,.\db\scripts\02_seed.sql --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json
```

```bash
# Dry run to validate inputs
./appian-deployment-cli deploy --package-zip-name ./artifacts/my_package.zip --name "My Deploy" --description "Testing" --rollback-on-failure --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --dry-run

# Real deployment with optional files
./appian-deployment-cli deploy --package-zip-name ./artifacts/my_package.zip --name "My Deploy" --customization-file ./configs/import.properties --admin-console-file ./configs/admin.zip --plugins-file ./plugins/plugins.zip --database-scripts ./db/scripts/01_schema.sql,./db/scripts/02_seed.sql --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json
```

### status (alias: get-deployment)
Check status of a deployment or export.
- Flags:
  - `--deployment-uuid <UUID>` (required)
  - `--kind <export|deployment>` (optional; default `deployment`)
- Example:
```powershell
./appian-deployment-cli.exe status --deployment-uuid 00000000-0000-0000-0000-000000000000 --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json
```

```bash
./appian-deployment-cli status --deployment-uuid 00000000-0000-0000-0000-000000000000 --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json
```

### results (alias: get-deployment-results)
Retrieve deployment results; optionally poll until terminal status.
- Flags:
  - `--deployment-uuid <UUID>` (required)
  - `--poll` (optional; waits until terminal status)
- Example:
```powershell
./appian-deployment-cli.exe results --deployment-uuid 00000000-0000-0000-0000-000000000000 --poll --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY --format json
```

```bash
./appian-deployment-cli results --deployment-uuid 00000000-0000-0000-0000-000000000000 --poll --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY" --format json
```

### monitor
Poll status at an interval until completion (export or deployment).
- Flags:
  - `--deployment-uuid <UUID>` (required)
  - `--kind <export|deployment>` (optional; default `deployment`)
  - `--interval-seconds <INT>` (default `10`)
  - `--timeout-seconds <INT>` (optional; default `3600` via code)
- Example:
```powershell
./appian-deployment-cli.exe monitor --deployment-uuid 00000000-0000-0000-0000-000000000000 --interval-seconds 15 --timeout-seconds 600 --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY
```

```bash
./appian-deployment-cli monitor --deployment-uuid 00000000-0000-0000-0000-000000000000 --interval-seconds 15 --timeout-seconds 600 --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY"
```

### download-package
Download an exported artifact by UUID.
- Flags:
  - `--deployment-uuid <UUID>` (required)
  - `--output <PATH>` (optional; defaults to `<UUID>.zip`)
  - `--overwrite` (optional)
- Example:
```powershell
./appian-deployment-cli.exe download-package --deployment-uuid 00000000-0000-0000-0000-000000000000 --output .\artifacts\export.zip --overwrite --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY
```

```bash
./appian-deployment-cli download-package --deployment-uuid 00000000-0000-0000-0000-000000000000 --output ./artifacts/export.zip --overwrite --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY"
```

### logs (feature-gated)
Retrieve deployment logs; stream with `--follow`.
- Flags:
  - `--deployment-uuid <UUID>` (required)
  - `--follow` (optional; stream)
  - `--tail <INT>` (optional; last N lines)
- Example:
```powershell
./appian-deployment-cli.exe logs --deployment-uuid 00000000-0000-0000-0000-000000000000 --tail 100 --base-url https://mysite.appiancloud.com --api-key $env:APPIAN_API_KEY
```

```bash
./appian-deployment-cli logs --deployment-uuid 00000000-0000-0000-0000-000000000000 --tail 100 --base-url https://mysite.appiancloud.com --api-key "$APPIAN_API_KEY"
```

## Output Notes
- `--format json` returns structured JSON for server-backed operations; `--dry-run` prints validation text.
- Non-zero exit codes indicate validation or runtime errors.

## Why CI/CD & DevOps Friendly (Windows, macOS, Linux)
- Cross-platform single binary per OS; no runtime installers required.
  - Windows: `appian-deployment-cli.exe`
  - macOS/Linux: `appian-deployment-cli`
- Predictable configuration with clear precedence: CLI overrides > environment variables > config file.
- Secret-friendly: use runner secrets and env vars; avoid printing tokens; prefer `--api-key` via env when possible.
- Machine-readable outputs with `--format json` enable robust parsing and gating.
- `--dry-run` validates inputs and fails fast without calling remote APIs.
- Non-zero exit codes map cleanly to pipeline fail/continue strategies.
- Optional features (e.g., `logs`) can be feature-gated at build time to minimize footprint.
- Integrates cleanly with GitHub Actions, Azure DevOps, GitLab CI, Jenkins, and self-hosted agents across Windows/macOS/Linux.

### Pipeline Examples (Cross-Platform)
- GitHub Actions matrix across Windows/macOS/Linux:
```yaml
jobs:
  deploy:
    strategy:
      matrix:
        os: [windows-latest, ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Set environment
        env:
          APPIAN_BASE_URL: https://mysite.appiancloud.com
          APPIAN_API_KEY: ${{ secrets.APPIAN_API_KEY }}
        run: echo "env set"
      - name: Export Package (dry run)
        shell: bash
        run: |
          BIN="./appian-deployment-cli"
          if [[ "${{ matrix.os }}" == windows* ]]; then BIN="./appian-deployment-cli.exe"; fi
          $BIN export --uuids "$UUIDS" --export-type package --name "$NAME" --description "$DESC" --dry-run --base-url "$APPIAN_BASE_URL" --api-key "$APPIAN_API_KEY" --format json

      - name: Deploy Package
        shell: bash
        run: |
          BIN="./appian-deployment-cli"
          if [[ "${{ matrix.os }}" == windows* ]]; then BIN="./appian-deployment-cli.exe"; fi
          $BIN deploy --package-zip-name ./artifacts/my_package.zip --name "$NAME" --description "$DESC" --rollback-on-failure --base-url "$APPIAN_BASE_URL" --api-key "$APPIAN_API_KEY" --format json > deploy.json
```

- Azure DevOps (Windows PowerShell):
```yaml
steps:
- powershell: |
    .\appian-deployment-cli.exe status --deployment-uuid $env:DEPLOYMENT_UUID --base-url $env:APPIAN_BASE_URL --api-key $(APPIAN_API_KEY) --format json | Set-Content status.json
  displayName: Check Deployment Status
```

- Parsing JSON
  - PowerShell (Windows):
    ```powershell
    $deployment = Get-Content deploy.json | ConvertFrom-Json
    if ($deployment.status -ne 'COMPLETED') { throw 'Deployment failed' }
    ```
  - Bash (macOS/Linux):
    ```bash
    status=$(jq -r '.status' deploy.json)
    test "$status" = "COMPLETED" || { echo "Deployment failed"; exit 1; }
    ```

## Security & Best Practices
- Store API keys in runner secrets or environment variables; avoid committing real secrets.
- Use `--config-file` per environment, or rely on global `APPIAN_*` env vars.

## Support
Open an issue or adapt the source to your needs.
For Appian API details, consult the Appian Deployment REST API v2 documentation.
