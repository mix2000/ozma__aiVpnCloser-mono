# ozma AI VPN Closer

Следит за VPN и, если он пропал, закрывает сессии **Claude Code**, **Claude**
(приложение), **Codex** и **Grok**, чтобы их запросы не ушли в интернет
напрямую. Как только VPN пропал — агенты сразу замораживаются, через заданное
время (по умолчанию 30 с) закрываются. Вернулся VPN раньше — продолжают работу.

Одна программа для macOS и Windows: окно и значок в меню-баре / трее.

## Установка

Готовые сборки — в GitHub Releases:

- Windows: `ai-vpn-closer-windows-x64-setup.exe`;
- macOS: `ai-vpn-closer-macos-arm64.dmg` (Apple Silicon) или `ai-vpn-closer-macos-x64.dmg`
  (Intel), рядом `.app.zip`. Сборка не подписана:
  при первом запуске — правый клик → «Открыть», либо
  `xattr -dr com.apple.quarantine "/Applications/ozma AI VPN Closer.app"`.

## Как пользоваться

1. «Наблюдение» → «Выбрать…»: отметить, что считать включённым VPN. Обычно —
   туннельный интерфейс VPN (`utun…`, WireGuard, Wintun) и пункт «трафик к
   агентам идёт через VPN»: он проверяет, что запросы к `api.anthropic.com`,
   `api.openai.com`, `api.x.ai` и др. на самом деле уходят в туннель.
   Есть также программы VPN (xray, sing-box, Happ, …) и системные VPN.
2. «Действие»: кого закрывать, замораживать ли сразу, через сколько закрыть.
3. Включить «Следить». При первом запуске включён режим «Только смотреть» —
   программа ничего не трогает, а пишет в «Журнал», что сделала бы. Когда
   убедились, что всё верно, — выключить его.

У туннельных интерфейсов в скобках — программа, которой принадлежит туннель:
«utun233 (sing-box)». Источники (только чтение, без sudo): сокет
`utun_control` у своих процессов (lsof), конфиг клиента из аргументов запуска
(sing-box, xray, clash/mihomo, OpenVPN — только поля туннеля), подключённый
системный VPN (`scutil --nc status`), описание адаптера на Windows. Если точно
не узнать, но подходящий клиент один — серым «(вероятно OpenVPN)».

Значок: зелёный — VPN есть, жёлтый — VPN пропал, идёт отсчёт, красный — агенты
закрыты, серый — наблюдение выключено. Закрыть окно — программа остаётся в
меню-баре / трее; выход — из меню значка. Автозапуск и уведомления — в
«Настройках».

Если программа упала или её убили (`kill -9`, `taskkill /F`), пока агенты
заморожены, следующий запуск их разморозит: список лежит в `frozen.json` рядом
с настройками. Пока VPN не подтверждён, они остаются замороженными (если
проверки ничего не говорят — до 30 с); если VPN нет — всё идёт как обычно: отсчёт и закрытие.

Отсчёт останавливает только подтверждённый VPN: если проверки во время отсчёта
перестали отвечать («неизвестно»), агенты всё равно будут закрыты.

На Windows мягкого закрытия нет: процессы завершаются сразу (`TerminateProcess`),
пауза перед принудительным закрытием действует только на macOS.

Настройки и журнал — в папке данных ОС: `~/Library/Application Support/ozma-ai-vpn-closer`,
`%APPDATA%\ozma-ai-vpn-closer` (`config.json`, `events.jsonl`, `frozen.json`). Другую папку
можно задать переменной `AI_VPN_CLOSER_DATA_DIR`.

## Разработка и проверки

```bash
export PATH=/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH
make install                 # npm install (хуки git, husky) + ui: npm ci
make dev                     # интерфейс в браузере с моком, http://127.0.0.1:47613
npm run check                # всё, что гоняет CI (= make check, scripts/check.sh)
make test                    # vitest + cargo test
make e2e                     # Playwright на моке, порт 47751 (сервер поднимает и гасит сам)
make guards                  # сторожа: файлы ≤ 500 строк, слои, реэкспорты, cfg только в platform/
make coverage                # покрытие UI (пороги в ui/vite.config.ts) и Rust (cargo llvm-cov)
node ui/node_modules/@tauri-apps/cli/tauri.js dev     # окно с настоящим ядром
```

В `ui/`: `npm run format` / `format:check` (prettier), `lint` (eslint), `typecheck`
(tsc -b), `test`, `test:coverage`, `test:e2e`. В браузере мок управляется из
консоли: `__mock.setVpn(false)`.

Хуки git (husky, ставятся `npm install` в корне):
- **pre-commit** — гард против `.env`/ключей в индексе, eslint + prettier по
  staged-файлам `ui/` (lint-staged), `cargo fmt --check`, если менялся Rust;
- **pre-push** — `ui`: lint, typecheck, test; Rust: clippy `-D warnings`,
  `cargo test` — только по затронутой части. Сети не требуют.

CI (`.github/workflows/check.yml`) — на macOS и Windows: format:check, lint,
typecheck, test:coverage, build UI, `cargo fmt`, `clippy`, `test` (вместе со
сторожами `crates/core/tests/guards.rs`); e2e — на macOS. Сборки и релизы — локально.

### Устройство

- `crates/core` — вся логика без GUI, по слоям (наружу не торчат, публичное
  API — `Engine` и `models`):
  - `models/` — типы контракта (serde), без ввода-вывода;
  - `services/` — проверки, агенты, автомат монитора, действия, владельцы
    туннелей (`owners/`), цвет и заголовок статуса — без ОС, только через trait'ы;
  - `platform/` — getifaddrs/scutil/lsof, GetAdaptersAddresses/rasdial,
    сигналы и NtSuspendProcess, резолвер, папка данных — за trait'ами
    (`NetProbe`, `DnsResolver`, `ProcessControl`, `ProcSource`, `OwnerProbe`),
    `#[cfg]` только здесь;
  - `storage/` — конфиг (только чтение/запись, проверка — в сервисе), журнал;
  - `engine/` — фасад `Engine` и корень композиции (`PlatformParts::os()` —
    единственное место, где собираются настоящие реализации), поток опроса.
  - Интеграционные тесты (`tests/`) берут внутренности из модуля `testing`
    под фичей `test-util`; действия — только над своими пустышками.
- `crates/app` — Tauri 2: `commands.rs` (проброс в `Engine`), `bridge.rs`
  (состояние → UI и трей, уведомления, автозапуск из подписки на настройки),
  `tray.rs`, `notifications.rs`, `window.rs`, `autostart.rs`, `logger.rs`,
  `status/`, `platform/`; `main.rs` — только сборка.
- `ui` — React 19 + antd 6 + React Query, по образцу CheerPortal:
  `common/types` (контракт ядра), `common/services/api` (транспорт, `apiService`,
  `queryKeys`), `common/services/theme`, `providers/`, `pages/main-page`
  (страница из четырёх сущностей), `entities/<сущность>/<фича>/`
  (`monitor/status`, `vpn/{watch,picker,owner}`, `agents/action`,
  `settings/{config,form}`, `events/journal`: компонент `<имя>.tsx` = имя
  файла, `hooks.ts`, `<сущность>.api.ts`, `components/`, тексты в `*.consts.ts`,
  стили в `*.module.scss`, тесты рядом), `mocks/` (мок бэкенда), `e2e/`.

Сторожа (`make guards`; фронт — `ui/src/guards.test.ts`, Rust —
`crates/core/tests/guards.rs`; тот же текст, что в ozma Session Search): файлы
≤ 500 строк, функции Rust и компоненты React ≤ 60 строк, `#[cfg(target_os…)]`
только в `platform/`, слои ядра, без `export *`, транспорт только через
`apiService`, кириллица только в `*.consts.ts`, между сущностями — только через
их `index.ts`, без inline `style`, имя файла = имя компонента.

## Релиз

Собирается на маке сразу для трёх платформ: macOS arm64 и x64 (`.dmg`,
`.app.zip`) и Windows x64 (NSIS-установщик; через `cargo-xwin`, `.msi` с мака
не собрать).

Нужно: rustup, Node 20+, `cargo install cargo-xwin`,
`brew install llvm lld makensis`, для публикации — `gh auth login`.

```bash
# 1. Поднять версию в crates/app/tauri.conf.json и Cargo.toml ([workspace.package]),
#    закоммитить.
scripts/release.sh --allow-dirty          # пробная сборка в dist/
scripts/release.sh --create-tag           # тег vX.Y.Z на HEAD + сборка
git push origin main vX.Y.Z
scripts/release.sh --publish              # сборка и выкладка в GitHub Release
```

Скрипт проверяет, что дерево чистое и тег `vX.Y.Z` указывает на HEAD. В `dist/`
кладёт `ai-vpn-closer-macos-arm64.dmg` / `.app.zip`,
`ai-vpn-closer-macos-x64.dmg` / `.app.zip`, `ai-vpn-closer-windows-x64-setup.exe`
и `SHA256SUMS.txt`. С `--publish` создаёт релиз (`--generate-notes`), а если он
уже есть — перезаливает файлы (`gh release upload --clobber`). `--only mac|win` —
собрать одну платформу.
