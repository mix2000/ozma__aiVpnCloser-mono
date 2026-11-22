#!/usr/bin/env bash
# Все проверки разом (как в CI): UI — формат, линт, типы, тесты с покрытием;
# Rust — fmt, clippy, тесты; сторожа. e2e — отдельно: make e2e.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
# Чужие переменные окружения сессии не должны влиять на проверки.
unset PORT

step() { printf '\n==> %s\n' "$*"; }

step "UI: format:check, lint, typecheck, test:coverage"
(cd ui && npm run format:check && npm run lint && npm run typecheck && npm run test:coverage)

step "Rust: fmt, clippy, test"
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace

step "Сторожа"
bash scripts/guards.sh

printf '\nВсё зелёное.\n'
