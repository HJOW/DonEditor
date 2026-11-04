#!/usr/bin/env bash
# 한 줄 주석
set -euo pipefail

NAME="Don Editor"
COUNT=3

greet() {
  local who="$1"
  echo "안녕하세요, ${who}!"
}

for i in $(seq 1 "$COUNT"); do
  if [ "$i" -gt 1 ] && [[ -n "$NAME" ]]; then
    greet "$NAME"
  fi
done

case "$1" in
  start) echo "시작" ;;
  *) echo "알 수 없음" ;;
esac
