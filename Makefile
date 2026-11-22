# Обёртка над командами проекта. Порты: dev UI — 47613, e2e — 47751 (не 38000/48000).
export PATH := /opt/homebrew/opt/rustup/bin:$(HOME)/.cargo/bin:/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$(PATH)

.PHONY: install check test e2e guards fmt coverage release dev

install:            ## зависимости UI и хуки git
	npm install
	cd ui && npm ci

check:              ## всё, что гоняет CI (без e2e)
	bash scripts/check.sh

test:               ## юнит- и интеграционные тесты UI и Rust
	cd ui && npm test
	cargo test --workspace

e2e:                ## Playwright на моке (сервер поднимает и гасит сам)
	cd ui && npm run test:e2e

guards:             ## сторожа: длина файлов, слои, cfg только в platform/
	bash scripts/guards.sh

fmt:                ## отформатировать всё
	cd ui && npm run format
	cargo fmt --all

coverage:           ## покрытие UI (пороги в vite.config.ts) и Rust (cargo llvm-cov)
	cd ui && npm run test:coverage
	cargo llvm-cov --workspace --summary-only

release:            ## локальная сборка релиза (см. scripts/release.sh)
	bash scripts/release.sh

dev:                ## интерфейс в браузере на моке
	cd ui && npm run dev
