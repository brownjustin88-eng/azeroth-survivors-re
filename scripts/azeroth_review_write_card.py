#!/usr/bin/env python3
"""Write ~/.config/pko/azeroth-survivors.connector.txt from token + public URL.

Never prints the bearer. Do not paste the card into Hall.
"""
from pathlib import Path
import json

CFG = Path.home() / ".config" / "pko"
TOKEN_FILE = CFG / "azeroth-survivors.connector.token"
URL_FILE = CFG / "azeroth-survivors.public.url"
CARD = CFG / "azeroth-survivors.connector.txt"
MCP_JSON = CFG / "azeroth-survivors.mcp.json"
LOCAL = "http://127.0.0.1:9830/mcp"


def main() -> int:
    tok = TOKEN_FILE.read_text().strip() if TOKEN_FILE.exists() else ""
    if not tok:
        raise SystemExit("missing token file — start the MCP first")
    url = LOCAL
    if URL_FILE.exists():
        public = URL_FILE.read_text().strip()
        if public:
            url = public
    card = (
        "Azeroth Survivors review connector (read-only source tree on grok-compile, "
        "loopback on TheCore :9830)\n"
        "Name: azeroth-survivors\n"
        f"URL: {url}\n"
        "Auth: request header\n"
        f"Authorization: Bearer {tok}\n"
        "First call: tools/list, then azeroth_summary\n"
        f"Linux Desktop local: {LOCAL}\n"
        "Quick-tunnel URL rotates if azeroth-survivors-mcp-funnel.service restarts.\n"
        f"Token file: {TOKEN_FILE}\n"
        "Do not paste this into Hall. Do not serve wow-data or MPQs.\n"
    )
    CARD.write_text(card)
    CARD.chmod(0o600)
    MCP_JSON.write_text(
        json.dumps(
            {
                "mcpServers": {
                    "azeroth-survivors": {
                        "type": "http",
                        "url": url,
                        "headers": {"Authorization": f"Bearer {tok}"},
                    }
                }
            },
            indent=2,
        )
        + "\n"
    )
    MCP_JSON.chmod(0o600)
    print(f"wrote {CARD} url={url}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
