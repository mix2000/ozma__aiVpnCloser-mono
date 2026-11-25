#!/usr/bin/env bash
# Дождаться зелёного CI (check.yml) для коммита; иначе — ненулевой код.
#
#   scripts/wait-ci.sh [sha]           # по умолчанию HEAD
#
# WAIT_CI_FIND_SEC — сколько ждать появления прогона (по умолчанию 300),
# WAIT_CI_POLL_SEC — пауза между опросами (по умолчанию 15).
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

SHA="$(git rev-parse "${1:-HEAD}^{commit}")"
POLL="${WAIT_CI_POLL_SEC:-15}"
FIND_UNTIL=$((SECONDS + ${WAIT_CI_FIND_SEC:-300}))

while :; do
  # Последний прогон коммита: «status conclusion url»; сбой сети — повторим.
  RUN="$(gh run list --workflow check.yml --commit "$SHA" --event push --limit 1 \
    --json status,conclusion,url --jq '.[0] // empty | "\(.status) \(.conclusion) \(.url)"' || true)"
  case "$RUN" in
    "completed success "*)
      echo "CI зелёный: ${RUN##* }"
      exit 0
      ;;
    "completed "*)
      echo "CI не прошёл: $RUN" >&2
      exit 1
      ;;
    "")
      if [ "$SECONDS" -ge "$FIND_UNTIL" ]; then
        echo "Нет прогона CI для ${SHA:0:7} (отправлен ли коммит в main?)" >&2
        exit 1
      fi
      ;;
    # Прогон есть — ждём, пока закончится.
    *) FIND_UNTIL=$((SECONDS + ${WAIT_CI_FIND_SEC:-300})) ;;
  esac
  sleep "$POLL"
done
