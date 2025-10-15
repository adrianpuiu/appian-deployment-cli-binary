# Appian Deployment CLI (Binary + Source)

A Windows-friendly command-line tool to automate Appian deployments via the Appian Deployment REST API v2. This repository intentionally includes only:

- src/  the Rust source code
- ppian-deployment-cli.exe  the prebuilt Windows x86_64 binary

Everything else (docs, tests, manifests) is omitted to keep distribution minimal. Use the binary directly, or adapt the source in your own workspace.

## Quick Start
Place ppian-deployment-cli.exe and ppian-config.toml together:

`	oml
base_url = "https://mysite.appiancloud.com"
api_key = "your-api-key-here"
timeout_seconds = 300
`

## Usage
PowerShell examples:

- Export (dry run)
`
./appian-deployment-cli.exe export --uuids 11111111-1111-1111-1111-111111111111 --export-type package --name "Sample Export" --description "Testing" --dry-run --format json
`

- Deploy (dry run)
`
./appian-deployment-cli.exe deploy --package-zip-name .\artifacts\my_package.zip --name "My Deploy" --description "Testing" --rollback-on-failure --dry-run --format json
`

- Inspect
`
./appian-deployment-cli.exe inspect --package .\artifacts\my_package.zip --format json
`

### Output Notes
- --format json emits structured output for server-backed operations; --dry-run uses validation logs.
- logs subcommand is gated behind --features logs when building from source.
