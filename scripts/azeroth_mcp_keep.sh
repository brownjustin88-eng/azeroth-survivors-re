#!/usr/bin/env bash
# Keep the read-only azeroth review MCP on grok-compile (57731778) and
# local-forward it to TheCore 127.0.0.1:9830. Silent unless state changes.
set -u
KEY="${AZEROTH_VAST_KEY:-$HOME/.ssh/vast_ai}"
PORT="${AZEROTH_VAST_PORT:-26116}"
HOST="${AZEROTH_VAST_HOST:-ssh1.vast.ai}"
LOCAL_PORT="${AZEROTH_MCP_PORT:-9830}"
REMOTE_ROOT="${AZEROTH_REMOTE_ROOT:-/root/azeroth-survivors-re}"
TOKEN_FILE="${AZEROTH_MCP_TOKEN_FILE:-$HOME/.config/pko/azeroth-survivors.connector.token}"
LOG="${AZEROTH_MCP_KEEP_LOG:-$HOME/.grok/long-running-background-tasks/azeroth_mcp_keep.log}"
SSH=(ssh -i "$KEY" -o IdentitiesOnly=yes -o BatchMode=yes -o ConnectTimeout=15
     -o ServerAliveInterval=30 -o ServerAliveCountMax=3 -o ExitOnForwardFailure=yes
     -p "$PORT" "root@$HOST")
STATE=down
mkdir -p "$(dirname "$LOG")"
log(){ echo "$(date -u '+%Y-%m-%dT%H:%M:%SZ') $*" | tee -a "$LOG"; }

remote_up(){
  "${SSH[@]}" 'python3 -c "import urllib.request; urllib.request.urlopen(\"http://127.0.0.1:9830/health\", timeout=3).read()"' >/dev/null 2>&1
}

start_remote(){
  "${SSH[@]}" "test -f $REMOTE_ROOT/scripts/azeroth_review_mcp.py || exit 2
    mkdir -p /root/.config/pko
    nohup env AZEROTH_ROOT=$REMOTE_ROOT AZEROTH_MCP_HOST=127.0.0.1 AZEROTH_MCP_PORT=9830 \
      AZEROTH_MCP_TOKEN_FILE=/root/.config/pko/azeroth-survivors.connector.token \
      python3 -u $REMOTE_ROOT/scripts/azeroth_review_mcp.py \
      >/root/.config/pko/azeroth-survivors.mcp.log 2>&1 &
    echo started \$!"
}

local_forward(){
  # kill a stale forward on this port
  pkill -f "ssh .* -L 127.0.0.1:${LOCAL_PORT}:127.0.0.1:9830 .* ${HOST}" 2>/dev/null || true
  ssh -i "$KEY" -o IdentitiesOnly=yes -o BatchMode=yes -o ConnectTimeout=15 \
      -o ServerAliveInterval=30 -o ServerAliveCountMax=3 -o ExitOnForwardFailure=yes \
      -N -L "127.0.0.1:${LOCAL_PORT}:127.0.0.1:9830" -p "$PORT" "root@$HOST" \
      >>"$LOG" 2>&1 &
  echo $!
}

# push token so Vast and TheCore share the same bearer
if [[ -f "$TOKEN_FILE" ]]; then
  scp -i "$KEY" -o IdentitiesOnly=yes -o BatchMode=yes -P "$PORT" \
    "$TOKEN_FILE" "root@$HOST:/root/.config/pko/azeroth-survivors.connector.token" >/dev/null 2>&1 || true
fi

FWD_PID=""
while true; do
  if remote_up; then
    if [[ "$STATE" != up ]]; then
      log "ACTION_REQUIRED: remote MCP health ok"
      STATE=up
    fi
  else
    if [[ "$STATE" != starting ]]; then
      log "remote MCP down — starting"
      STATE=starting
    fi
    start_remote >>"$LOG" 2>&1 || log "start_remote failed rc=$?"
    sleep 2
  fi
  if ! kill -0 "${FWD_PID:-0}" 2>/dev/null; then
    FWD_PID=$(local_forward)
    log "local-forward pid=$FWD_PID :${LOCAL_PORT}"
  fi
  sleep 20
done
