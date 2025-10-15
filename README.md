# Appian Deployment CLI (Binary + Source)

A Windows-friendly command-line tool to automate Appian deployments via the Appian Deployment REST API v2. This repository intentionally includes only:

- `src/`  the Rust source code
- `appian-deployment-cli.exe`  the prebuilt Windows x86_64 binary

Everything else (docs, tests, manifests) is omitted to keep distribution minimal. Use the binary directly, or adapt the source in your own workspace.

## Key Features
- Export Appian applications or packages
- Inspect packages pre-deployment
- Deploy packages and monitor status
- Retrieve deployment and inspection results
- Optional `logs` subcommand (gated behind the `logs` feature)
- `--format json` for machine-readable outputs (non-dry-run paths)

## Requirements
- Windows 10/11 (x86_64)
- An Appian environment with API key access

## Quick Start
Place `appian-deployment-cli.exe` and `appian-config.toml` together:

```toml
base_url = "https://mysite.appiancloud.com"
api_key = "your-api-key-here"
timeout_seconds = 300
```

Environment variables (optional): `APPIAN_BASE_URL`, `APPIAN_API_KEY`, `APPIAN_TIMEOUT_SECONDS`.

## Usage
PowerShell examples:

- Export (dry run)
```powershell
./appian-deployment-cli.exe export --uuids 11111111-1111-1111-1111-111111111111 --export-type package --name "Sample Export" --description "Testing" --dry-run --format json
```

- Export (application, dry run, multiple UUIDs)
```powershell
./appian-deployment-cli.exe export --uuids 11111111-1111-1111-1111-111111111111,22222222-2222-2222-2222-222222222222 --export-type application --name "My App Export" --description "Testing multi UUID" --dry-run --format json
```

- Deploy (dry run)
```powershell
./appian-deployment-cli.exe deploy --package-zip-name .\artifacts\my_package.zip --name "My Deploy" --description "Testing" --rollback-on-failure --dry-run --format json
```

- Inspect
```powershell
./appian-deployment-cli.exe inspect --package .\artifacts\my_package.zip --format json
```

- Get packages
```powershell
./appian-deployment-cli.exe get-packages --format json
```

- Status
```powershell
./appian-deployment-cli.exe status --deployment-uuid 00000000-0000-0000-0000-000000000000 --format json
```

### Output Notes
- `--format json` emits structured output for server-backed operations; `--dry-run` uses validation logs.
- `logs` subcommand is gated behind `--features logs` when building from source.

## Why CI/CD & DevOps Friendly
- Single binary on Windows runners; no runtime or installer dependencies.
- Predictable configuration via `appian-config.toml` and environment overrides; secret-friendly.
- Machine-readable outputs with `--format json` for reliable parsing in pipelines.
- `--dry-run` validation to fail fast before touching remote systems.
- Clear non-zero exit on errors; easy to gate steps, notify, and roll back.
- Optional features (e.g., `logs`) gated at build time to keep footprint minimal.
- Works on GitHub Actions, Azure DevOps, GitLab CI, and self-hosted Windows agents.

### Pipeline Examples
- GitHub Actions (Windows):
```yaml
- name: Export Package (dry run)
  run: .\\appian-deployment-cli.exe export --uuids $env:UUIDS --export-type package --name "$env:NAME" --description "$env:DESC" --dry-run --format json

- name: Deploy Package
  run: .\\appian-deployment-cli.exe deploy --package-zip-name .\\artifacts\\my_package.zip --name "$env:NAME" --description "$env:DESC" --rollback-on-failure --format json | Out-File deploy.json
```

- Azure DevOps (PowerShell):
```powershell
.\appian-deployment-cli.exe status --deployment-uuid $env:DEPLOYMENT_UUID --format json | Set-Content status.json
```

- Parsing JSON (PowerShell):
```powershell
$deployment = Get-Content deploy.json | ConvertFrom-Json
if ($deployment.status -ne 'COMPLETED') { throw 'Deployment failed' }
```

## Security & Best Practices
- Prefer storing secrets in environment variables or secret managers.
- Avoid committing configuration files with real secrets.

## Support
Open an issue or adapt the source to your needs. For Appian API details, consult the Appian Deployment REST API v2 documentation.
