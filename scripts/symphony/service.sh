#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
case "${1:-status}" in
  start) docker compose up -d --no-build --wait ;;
  stop) docker compose stop ;;
  restart) docker compose restart ;;
  status) docker compose ps ;;
  logs) docker compose logs --tail 100 symphony ;;
  *) echo "Usage: $0 {start|stop|restart|status|logs}" >&2; exit 2 ;;
esac
