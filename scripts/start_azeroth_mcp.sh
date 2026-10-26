#!/bin/bash
# Start (or restart) the read-only review MCP on this box. Run on grok-compile.
set -euo pipefail
ROOT="${AZEROTH_ROOT:-/root/azeroth-survivors-re}"
LOG="${AZEROTH_MCP_LOG:-/root/.config/pko/azeroth-survivors.mcp.log}"
TOKEN="${AZEROTH_MCP_TOKEN_FILE:-/root/.config/pko/azeroth-survivors.connector.token}"
mkdir -p "$(dirname "$LOG")"
# restart by pid file, not by matching this script's argv
PIDFILE=/root/.config/pko/azeroth-survivors.mcp.pid
if [ -f "$PIDFILE" ]; then
  old=$(cat "$PIDFILE" || true)
  if [ -n "${old:-}" ] && kill -0 "$old" 2>/dev/null; then
    kill "$old" || true
    sleep 1
  fi
fi
export PATH="/root/.local/bin:$PATH"
export AZEROTH_ROOT="$ROOT"
export AZEROTH_MCP_HOST=127.0.0.1
export AZEROTH_MCP_PORT=9830
export AZEROTH_MCP_TOKEN_FILE="$TOKEN"
nohup python3 -u "$ROOT/scripts/azeroth_review_mcp.py" >"$LOG" 2>&1 &
echo $! > "$PIDFILE"
sleep 2
python3 - <<'PY'
import urllib.request, sys
try:
    print(urllib.request.urlopen("http://127.0.0.1:9830/health", timeout=5).read().decode())
except Exception as e:
    print("health-fail", e)
    sys.exit(1)
PY
tail -20 "$LOG"
