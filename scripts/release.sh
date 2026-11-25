#!/usr/bin/env bash
# Локальная сборка релиза ozma AI VPN Closer для macOS (arm64, x64) и Windows x64.
#
#   scripts/release.sh                 # собрать всё в dist/
#   scripts/release.sh --publish       # собрать, дождаться CI и выложить в GitHub Release vX.Y.Z
#   scripts/release.sh --create-tag    # создать тег vX.Y.Z на HEAD, если его нет
#   scripts/release.sh --allow-dirty   # собрать из незакоммиченного дерева (без --publish)
#   scripts/release.sh --only mac|win  # только одна платформа
#
# Версия — из crates/app/tauri.conf.json. Windows собирается с мака через
# cargo-xwin (clang-cl, lld-link, llvm-rc) и makensis: только NSIS-установщик,
# .msi с мака не собрать.
set -euo pipefail

export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$PATH"
# Лицензия MSVC CRT/SDK, которые скачивает cargo-xwin.
export XWIN_ACCEPT_LICENSE=1
# Чужие переменные из окружения сессии не должны влиять на сборку.
unset PORT

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PUBLISH=0
CREATE_TAG=0
ALLOW_DIRTY=0
ONLY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --publish) PUBLISH=1 ;;
    --create-tag) CREATE_TAG=1 ;;
    --allow-dirty) ALLOW_DIRTY=1 ;;
    --only)
      shift
      ONLY="${1:-}"
      [ "$ONLY" = mac ] || [ "$ONLY" = win ] || { echo "--only: mac или win" >&2; exit 2; }
      ;;
    -h | --help)
      sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "Неизвестный аргумент: $1" >&2
      exit 2
      ;;
  esac
  shift
done

die() {
  echo "Ошибка: $*" >&2
  exit 1
}
step() { printf '\n==> %s\n' "$*"; }

[ "$(uname -s)" = Darwin ] || die "скрипт рассчитан на macOS"
for tool in node npm cargo rustup shasum ditto; do
  command -v "$tool" >/dev/null || die "нет $tool"
done

VERSION="$(node -p "require('./crates/app/tauri.conf.json').version")"
CARGO_VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version *= *"\(.*\)"/\1/p' Cargo.toml)"
[ "$VERSION" = "$CARGO_VERSION" ] ||
  die "версия в tauri.conf.json ($VERSION) не совпадает с Cargo.toml ($CARGO_VERSION)"
TAG="v$VERSION"
echo "Версия $VERSION, тег $TAG"

# ------------------------------------------------------------------ проверки git
if [ -n "$(git status --porcelain)" ]; then
  if [ "$ALLOW_DIRTY" = 1 ] && [ "$PUBLISH" = 0 ]; then
    echo "Внимание: в дереве есть незакоммиченные изменения (--allow-dirty)"
  else
    die "в дереве есть незакоммиченные изменения (для пробной сборки: --allow-dirty без --publish)"
  fi
fi

HEAD_SHA="$(git rev-parse HEAD)"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  TAG_SHA="$(git rev-list -n1 "$TAG")"
  if [ "$TAG_SHA" != "$HEAD_SHA" ]; then
    if [ "$ALLOW_DIRTY" = 1 ] && [ "$PUBLISH" = 0 ]; then
      echo "Внимание: тег $TAG указывает не на HEAD (--allow-dirty)"
    else
      die "тег $TAG указывает на ${TAG_SHA:0:7}, а HEAD — ${HEAD_SHA:0:7}; поднимите версию"
    fi
  fi
elif [ "$CREATE_TAG" = 1 ]; then
  git tag -a "$TAG" -m "$TAG"
  echo "Создан тег $TAG на ${HEAD_SHA:0:7}"
elif [ "$PUBLISH" = 1 ]; then
  die "нет тега $TAG (создать на HEAD: --create-tag)"
else
  echo "Внимание: тега $TAG нет — собираю без него"
fi

if [ "$PUBLISH" = 1 ]; then
  command -v gh >/dev/null || die "нет gh"
  gh auth status >/dev/null 2>&1 || die "gh не авторизован"
  git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null ||
    die "тег $TAG не отправлен: git push origin $TAG"
fi

# ------------------------------------------------------------------ сборка
DIST="$ROOT/dist"
rm -rf "$DIST"
mkdir -p "$DIST"

TAURI=(node "$ROOT/ui/node_modules/@tauri-apps/cli/tauri.js")
# UI собираем один раз ниже — beforeBuildCommand не нужен.
NO_UI_BUILD='{"build":{"beforeBuildCommand":""}}'

step "UI: npm ci, build"
(cd ui && npm ci --no-audit --no-fund && npm run build)

build_mac() {
  local target="$1" name="$2"
  step "macOS $name ($target)"
  rustup target add "$target" >/dev/null
  # CI=true: create-dmg без AppleScript в Finder (иначе таймаут без GUI-доступа).
  CI=true "${TAURI[@]}" build --target "$target" --bundles app,dmg --config "$NO_UI_BUILD"
  local bundle="$ROOT/target/$target/release/bundle"
  local app dmg
  app="$(find "$bundle/macos" -maxdepth 1 -name '*.app' | head -n1)"
  dmg="$(find "$bundle/dmg" -maxdepth 1 -name '*.dmg' | head -n1)"
  [ -n "$app" ] && [ -n "$dmg" ] || die "не найдены .app/.dmg для $target"
  cp "$dmg" "$DIST/ai-vpn-closer-$name.dmg"
  ditto -c -k --sequesterRsrc --keepParent "$app" "$DIST/ai-vpn-closer-$name.app.zip"
  file "$app/Contents/MacOS/ai-vpn-closer"
}

build_win() {
  local target=x86_64-pc-windows-msvc
  step "Windows x64 ($target, cargo-xwin)"
  for tool in cargo-xwin clang-cl lld-link llvm-rc llvm-lib makensis; do
    command -v "$tool" >/dev/null || die "нет $tool (нужно для сборки под Windows)"
  done
  rustup target add "$target" >/dev/null
  # makensis 3.13 (Homebrew) падает с bad_alloc без UTF-8 локали.
  LC_ALL=en_US.UTF-8 LANG=en_US.UTF-8 "${TAURI[@]}" build --runner cargo-xwin --target "$target" --bundles nsis --config "$NO_UI_BUILD"
  local exe
  exe="$(find "$ROOT/target/$target/release/bundle/nsis" -maxdepth 1 -name '*-setup.exe' | head -n1)"
  [ -n "$exe" ] || die "не найден NSIS-установщик"
  cp "$exe" "$DIST/ai-vpn-closer-windows-x64-setup.exe"
  file "$ROOT/target/$target/release/ai-vpn-closer.exe" "$DIST/ai-vpn-closer-windows-x64-setup.exe"
}

if [ "$ONLY" != win ]; then
  build_mac aarch64-apple-darwin macos-arm64
  build_mac x86_64-apple-darwin macos-x64
fi
if [ "$ONLY" != mac ]; then
  build_win
fi

step "SHA256SUMS.txt"
(cd "$DIST" && shasum -a 256 ai-vpn-closer-* >SHA256SUMS.txt && cat SHA256SUMS.txt)
ls -lh "$DIST"

# ------------------------------------------------------------------ публикация
if [ "$PUBLISH" = 0 ]; then
  echo
  echo "Собрано в dist/. Выложить: scripts/release.sh --publish"
  exit 0
fi

step "CI для ${HEAD_SHA:0:7}"
bash "$ROOT/scripts/wait-ci.sh" "$HEAD_SHA" || die "CI не подтвердил коммит — не публикую (файлы в dist/)"

step "GitHub Release $TAG"
if gh release view "$TAG" >/dev/null 2>&1; then
  gh release upload "$TAG" "$DIST"/* --clobber
else
  gh release create "$TAG" "$DIST"/* --title "$TAG" --verify-tag --generate-notes
fi
gh release view "$TAG" --json url --jq .url
