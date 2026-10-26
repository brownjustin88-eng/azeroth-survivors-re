#!/usr/bin/env python3
"""Read-only streamable-HTTP MCP over one Azeroth Survivors tree.

Jailed to AZEROTH_ROOT. No writes, no shell, no MPQ / wow-data / target / .so.
Bind loopback. Public door is a funnel on TheCore after an SSH local-forward.

Auth: Authorization: Bearer from AZEROTH_MCP_TOKEN_FILE.
"""
from __future__ import annotations

import json
import os
import re
import secrets
import subprocess
import time
from pathlib import Path

from mcp.server.fastmcp import FastMCP
from mcp.server.transport_security import TransportSecuritySettings

ROOT = Path(os.environ.get("AZEROTH_ROOT", "/root/azeroth-survivors-re")).resolve()
BIND_HOST = os.environ.get("AZEROTH_MCP_HOST", "127.0.0.1")
BIND_PORT = int(os.environ.get("AZEROTH_MCP_PORT", "9830"))
TOKEN_FILE = Path(
    os.environ.get(
        "AZEROTH_MCP_TOKEN_FILE",
        str(Path.home() / ".config" / "pko" / "azeroth-survivors.connector.token"),
    )
)
LOG_FILE = Path(
    os.environ.get(
        "AZEROTH_MCP_LOG_FILE",
        str(Path.home() / ".config" / "pko" / "azeroth-survivors.tools.jsonl"),
    )
)
READ_CAP = 80_000
GREP_CAP = 80
TREE_CAP = 400
MAX_FILE = 2_000_000

DENY_DIR = re.compile(
    r"(?ix)^(target|wow-data|\.git/objects|jniLibs|\.gradle|node_modules|__pycache__)$"
)
DENY_FILE = re.compile(r"(?ix)\.(mpq|so|apk|a|o|rlib|rmeta|wasm|bin)$")
SECRET_NAME = re.compile(
    r"(?ix)(token|secret|password|passwd|credential|\.env|id_rsa|id_ed25519|vast_ai|\.pem)$"
)


def load_token() -> str:
    TOKEN_FILE.parent.mkdir(parents=True, exist_ok=True)
    if TOKEN_FILE.exists():
        tok = TOKEN_FILE.read_text().strip()
        if tok:
            return tok
    tok = secrets.token_hex(32)
    old = os.umask(0o077)
    try:
        TOKEN_FILE.write_text(tok + "\n")
    finally:
        os.umask(old)
    TOKEN_FILE.chmod(0o600)
    return tok


TOKEN = load_token()


def jlog(tool: str, **fields) -> None:
    rec = {"ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "tool": tool}
    rec.update(fields)
    LOG_FILE.parent.mkdir(parents=True, exist_ok=True)
    new = not LOG_FILE.exists()
    with LOG_FILE.open("a") as f:
        f.write(json.dumps(rec, ensure_ascii=False, default=str) + "\n")
    if new:
        LOG_FILE.chmod(0o600)


def resolve_read(rel: str) -> Path:
    raw = (rel or ".").strip() or "."
    if raw.startswith("~") or raw.startswith("/"):
        cand = Path(raw)
    else:
        cand = ROOT / raw
    real = Path(os.path.realpath(cand))
    root = Path(os.path.realpath(ROOT))
    if real != root and not real.is_relative_to(root):
        raise ValueError(f"refused: {rel!r} is outside the review tree")
    for part in real.relative_to(root).parts:
        if DENY_DIR.match(part) or SECRET_NAME.search(part):
            raise ValueError(f"refused: {part!r} is outside the review surface")
    if real.is_file() and DENY_FILE.search(real.name):
        raise ValueError(f"refused: {real.name} is a binary/asset, not source")
    return real


def clip(s: str, cap: int = READ_CAP) -> str:
    if len(s) <= cap:
        return s
    return s[:cap] + f"\n… truncated {len(s) - cap} chars\n"


mcp = FastMCP(
    "azeroth-survivors",
    instructions=(
        "Read-only review MCP for Azeroth Survivors (Rust/Bevy 0.18, house APK "
        "house.azeroth.survivors). Root is the cloned source tree. No writes. "
        "No MPQs, no target/, no .so. Start with azeroth_summary, then azeroth_tree "
        "on crates/benilla-app/src/survivors and android/. Sitting questions: "
        "upgrade-card lag on 1/2/3, portrait HUD sizing, landscape rotate, missing "
        "options menu, missing 3D M2 bodies on Adreno, overlay = remapped WASD/Space/F/C/Esc. "
        "Do not pack MPQs. Do not Magisk SM-S908U or kirkwood (neither unlocks). "
        "Keep recommendations in the tree under docs/."
    ),
    host=BIND_HOST,
    port=BIND_PORT,
    streamable_http_path="/mcp",
    stateless_http=True,
    json_response=False,
    transport_security=TransportSecuritySettings(enable_dns_rebinding_protection=False),
)


@mcp.tool(
    name="azeroth_summary",
    description="Crate map, sitting facts, and the files that own HUD/cards/menu/Android.",
)
def azeroth_summary() -> str:
    jlog("azeroth_summary")
    head = ""
    try:
        head = subprocess.check_output(
            ["git", "-C", str(ROOT), "log", "-1", "--format=%H %ci %s"],
            text=True,
            timeout=8,
        ).strip()
    except Exception as e:
        head = f"(git unavailable: {e})"
    status = ""
    try:
        status = subprocess.check_output(
            ["git", "-C", str(ROOT), "status", "-sb"],
            text=True,
            timeout=8,
        ).strip()
    except Exception:
        status = "(no git status)"
    return clip(
        "\n".join(
            [
                f"root={ROOT}",
                f"HEAD {head}",
                status,
                "",
                "Play: native GameActivity APK package house.azeroth.survivors, lib warcraft_survivors.",
                "Engine: crates/benilla-app survivors mode on Bevy 0.18.1 / wgpu 27. Stay Rust.",
                "Data: MPQs beside the APK at /sdcard/azeroth-survivors/Data. Never public.",
                "",
                "Own these files first:",
                "  crates/benilla-app/src/survivors/cards.rs   deal + animate (CARD_W 236, CARD_H 330, BoxShadow, gradients, per-frame tilt)",
                "  crates/benilla-app/src/survivors/hud.rs     768-tall UiScale, OfferRow, LevelUpRoot 1400px Rays",
                "  crates/benilla-app/src/survivors/menu.rs    1.12 create/select glue, TOWER_H 590, no Options frame",
                "  crates/benilla-app/src/survivors/touch.rs   overlay stick/JUMP/F/C/II",
                "  crates/benilla-app/src/touch_pointer.rs     last finger held as cursor",
                "  android/app/src/main/AndroidManifest.xml   screenOrientation fullUser",
                "  CONTROLS.md                                Esc pause; 1/2/3 cards; no Options key",
                "  docs/ANDROID.md docs/ANDROID_SITTING.md docs/ANDROID_REVIEW.md docs/REVIEW_ASK.md",
                "",
                "Phone trial: gameplay works. Cards 1/2/3 lag. Portrait HUD oversized. Landscape not felt on the old portrait-locked APK. Options menu not visible because survivors never wires 1.12 GameMenuFrame.",
            ]
        )
    )


@mcp.tool(
    name="azeroth_tree",
    description="List files under a relative path. Skips target/, wow-data/, binaries.",
)
def azeroth_tree(path: str = ".", max_depth: int = 3) -> str:
    jlog("azeroth_tree", path=path, max_depth=max_depth)
    base = resolve_read(path)
    if not base.exists():
        return f"missing: {path}"
    max_depth = max(1, min(int(max_depth), 8))
    root = Path(os.path.realpath(ROOT))
    lines: list[str] = []
    if base.is_file():
        st = base.stat()
        return f"{base.relative_to(root)}  {st.st_size}B"

    for dirpath, dirnames, filenames in os.walk(base):
        rel_dir = Path(dirpath).resolve().relative_to(root)
        depth = 0 if rel_dir == Path(".") else len(rel_dir.parts)
        dirnames[:] = sorted(
            d
            for d in dirnames
            if not DENY_DIR.match(d) and not d.startswith(".") and d != ".git"
        )
        if depth >= max_depth:
            dirnames.clear()
            continue
        shown = sorted(f for f in filenames if not DENY_FILE.search(f) and not SECRET_NAME.search(f))
        prefix = "." if rel_dir == Path(".") else str(rel_dir)
        if shown:
            lines.append(f"{prefix}/")
            for name in shown[:80]:
                p = Path(dirpath) / name
                try:
                    sz = p.stat().st_size
                except OSError:
                    sz = -1
                lines.append(f"  {name}  {sz}B")
            if len(shown) > 80:
                lines.append(f"  … {len(shown) - 80} more")
        if len(lines) >= TREE_CAP:
            lines.append("… tree cap")
            break
    return clip("\n".join(lines) or "(empty)")


@mcp.tool(
    name="azeroth_read",
    description="Read a source file. path is relative to the tree. 1-indexed lines.",
)
def azeroth_read(path: str, start_line: int = 1, limit: int = 200) -> str:
    jlog("azeroth_read", path=path, start_line=start_line, limit=limit)
    real = resolve_read(path)
    if not real.is_file():
        return f"not a file: {path}"
    if real.stat().st_size > MAX_FILE:
        return f"too large: {path} ({real.stat().st_size} bytes)"
    start_line = max(1, int(start_line))
    limit = max(1, min(int(limit), 400))
    text = real.read_text(errors="replace")
    lines = text.splitlines()
    end = min(len(lines), start_line - 1 + limit)
    chunk = lines[start_line - 1 : end]
    body = "\n".join(f"{start_line + i:>5}→{row}" for i, row in enumerate(chunk))
    return clip(f"{path} lines {start_line}-{end} of {len(lines)}\n{body}")


@mcp.tool(
    name="azeroth_grep",
    description="Search source. glob like *.rs. path is a subdirectory. Regex is Python.",
)
def azeroth_grep(pattern: str, path: str = ".", glob: str = "*.rs", max_matches: int = 60) -> str:
    jlog("azeroth_grep", pattern=pattern, path=path, glob=glob)
    base = resolve_read(path)
    try:
        rx = re.compile(pattern)
    except re.error as e:
        return f"bad regex: {e}"
    max_matches = max(1, min(int(max_matches), GREP_CAP))
    hits: list[str] = []
    root = Path(os.path.realpath(ROOT))
    g = glob or "*"
    for p in base.rglob(g):
        if not p.is_file():
            continue
        try:
            rel = p.resolve().relative_to(root)
        except ValueError:
            continue
        if any(DENY_DIR.match(part) for part in rel.parts):
            continue
        if DENY_FILE.search(p.name) or SECRET_NAME.search(p.name):
            continue
        try:
            if p.stat().st_size > MAX_FILE:
                continue
            text = p.read_text(errors="replace")
        except OSError:
            continue
        for i, line in enumerate(text.splitlines(), 1):
            if rx.search(line):
                hits.append(f"{rel}:{i}:{line[:240]}")
                if len(hits) >= max_matches:
                    return clip("\n".join(hits))
    return clip("\n".join(hits) or "(no matches)")


@mcp.tool(name="azeroth_git_log", description="Recent commits in the review clone.")
def azeroth_git_log(n: int = 15) -> str:
    jlog("azeroth_git_log", n=n)
    n = max(1, min(int(n), 40))
    try:
        out = subprocess.check_output(
            ["git", "-C", str(ROOT), "log", f"-{n}", "--oneline", "--decorate"],
            text=True,
            timeout=8,
        )
    except Exception as e:
        return f"git log failed: {e}"
    return clip(out)


@mcp.tool(name="azeroth_git_status", description="git status -sb plus untracked (no binaries).")
def azeroth_git_status() -> str:
    jlog("azeroth_git_status")
    try:
        out = subprocess.check_output(
            ["git", "-C", str(ROOT), "status", "-sb", "--untracked-files=all"],
            text=True,
            timeout=8,
        )
    except Exception as e:
        return f"git status failed: {e}"
    keep = []
    for line in out.splitlines():
        if DENY_FILE.search(line) or "wow-data" in line or "/target/" in line:
            continue
        keep.append(line)
    return clip("\n".join(keep))


class BearerGate:
    def __init__(self, app, token: str):
        self.app = app
        self.token = token

    async def __call__(self, scope, receive, send):
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return
        path = scope.get("path") or ""
        method = scope.get("method") or "GET"
        if method == "OPTIONS":
            await send(
                {
                    "type": "http.response.start",
                    "status": 204,
                    "headers": [
                        (b"allow", b"GET, POST, DELETE, OPTIONS"),
                        (b"content-length", b"0"),
                    ],
                }
            )
            await send({"type": "http.response.body", "body": b""})
            return
        if path in ("/health", "/"):
            body = json.dumps({"ok": True, "server": "azeroth-survivors", "root": str(ROOT)}).encode()
            await send(
                {
                    "type": "http.response.start",
                    "status": 200,
                    "headers": [
                        (b"content-type", b"application/json"),
                        (b"content-length", str(len(body)).encode()),
                    ],
                }
            )
            await send({"type": "http.response.body", "body": body})
            return
        headers = {
            k.decode("latin1").lower(): v.decode("latin1") for k, v in (scope.get("headers") or [])
        }
        raw = headers.get("authorization", "").strip()
        tok = raw[7:].strip() if raw.lower().startswith("bearer ") else raw
        if not tok:
            body = b'{"error":"missing_authorization","hint":"Authorization: Bearer from the azeroth-survivors connector card"}\n'
            await send(
                {
                    "type": "http.response.start",
                    "status": 401,
                    "headers": [
                        (b"content-type", b"application/json"),
                        (b"www-authenticate", b"Bearer"),
                        (b"content-length", str(len(body)).encode()),
                    ],
                }
            )
            await send({"type": "http.response.body", "body": body})
            return
        if tok != self.token:
            body = b'{"error":"token_rejected"}\n'
            await send(
                {
                    "type": "http.response.start",
                    "status": 403,
                    "headers": [
                        (b"content-type", b"application/json"),
                        (b"content-length", str(len(body)).encode()),
                    ],
                }
            )
            await send({"type": "http.response.body", "body": body})
            return
        await self.app(scope, receive, send)


def main() -> None:
    import uvicorn

    if not ROOT.is_dir():
        raise SystemExit(f"AZEROTH_ROOT missing: {ROOT}")
    inner = mcp.streamable_http_app()
    app = BearerGate(inner, TOKEN)
    print(
        f"azeroth-survivors mcp bind=http://{BIND_HOST}:{BIND_PORT}/mcp root={ROOT} token_file={TOKEN_FILE}",
        flush=True,
    )
    uvicorn.run(app, host=BIND_HOST, port=BIND_PORT, log_level="info")


if __name__ == "__main__":
    main()
