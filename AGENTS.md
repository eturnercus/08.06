# AGENTS.md

Guidance for cloud agents working in this repository.

## Repository status

This repository is currently a **minimal placeholder**. It contains a single tracked file (`1` with the literal content `1`). There is no application source code, package manifest, Dockerfile, test suite, or README with run instructions.

When real application code is added, update this file with service-specific startup, lint, and test commands.

## Cursor Cloud specific instructions

### Services

| Service | Required | Notes |
|---------|----------|-------|
| — | — | No services are defined in this repository yet. |

### Environment

The cloud VM provides standard development tooling out of the box:

- **Git** — repository is cloned and ready
- **Node.js** — available via nvm (`node`, `npm`, `pnpm`, `yarn`)
- **Python 3.12** — `python3` and `pip` available

No dependency installation is required for the current repository contents. The VM update script is a no-op (`true`).

### Lint / test / run

Not applicable until application code and tooling are added. When a stack is introduced, document the commands here (for example `npm run lint`, `npm test`, `npm run dev`) and keep the VM update script limited to dependency refresh only.
