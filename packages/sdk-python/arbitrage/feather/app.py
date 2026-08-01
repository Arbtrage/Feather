from __future__ import annotations

import json
import os
import threading
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from .client import FeatherClient
from .worker import Handler, Worker

EnqueuePayload = bytes | str | dict[str, Any]


@dataclass
class FeatherUiOptions:
    enabled: bool = False
    port: int = 3001
    admin_url: str = "http://localhost:8080"
    open_browser: bool = False


class _UiHandler(SimpleHTTPRequestHandler):
    admin_url: str = "http://localhost:8080"

    def do_GET(self) -> None:
        if self.path.split("?", 1)[0] == "/config.js":
            body = f"window.__FEATHER_CONFIG__ = {json.dumps({'adminUrl': self.admin_url})};"
            encoded = body.encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/javascript; charset=utf-8")
            self.send_header("Content-Length", str(len(encoded)))
            self.end_headers()
            self.wfile.write(encoded)
            return
        if self.path in ("", "/"):
            self.path = "/index.html"
        return super().do_GET()

    def log_message(self, format: str, *args: Any) -> None:
        return


class FeatherApp:
    """
    Celery-style app: define tasks, enqueue with .delay(), run worker in-process
    via start_embedded() — no separate worker deployment required.
    """

    def __init__(
        self,
        *,
        address: str | None = None,
        queue: str = "default",
        worker_id: str | None = None,
        queues: list[str] | None = None,
        ui: FeatherUiOptions | dict[str, Any] | None = None,
    ) -> None:
        self._client = FeatherClient(address)
        self._default_queue = queue
        self._worker = Worker(
            address=address,
            worker_id=worker_id,
            queues=queues or [queue],
        )
        self._stop: Callable[[], Awaitable[None]] | None = None
        self._ui_server: ThreadingHTTPServer | None = None
        self._ui_thread: threading.Thread | None = None
        if isinstance(ui, dict):
            self._ui = FeatherUiOptions(
                enabled=bool(ui.get("enabled", False)),
                port=int(ui.get("port", 3001)),
                admin_url=str(
                    ui.get("admin_url")
                    or os.environ.get("FEATHER_ADMIN_URL")
                    or "http://localhost:8080"
                ),
                open_browser=bool(ui.get("open_browser", False)),
            )
        elif ui is None:
            self._ui = FeatherUiOptions(
                admin_url=os.environ.get("FEATHER_ADMIN_URL", "http://localhost:8080")
            )
        else:
            self._ui = ui

    def task(self, name: str, handler: Handler) -> FeatherApp:
        self._worker.task(name, handler)
        return self

    def _encode_payload(self, payload: EnqueuePayload | None) -> bytes:
        if payload is None:
            return b""
        if isinstance(payload, bytes):
            return payload
        if isinstance(payload, str):
            return payload.encode()
        return json.dumps(payload).encode()

    def enqueue(
        self,
        name: str,
        payload: EnqueuePayload | None = None,
        *,
        queue: str | None = None,
        priority: int = 0,
    ) -> str:
        return self._client.enqueue(
            name,
            queue=queue or self._default_queue,
            payload=self._encode_payload(payload),
            priority=priority,
        )

    def delay(
        self,
        name: str,
        payload: EnqueuePayload | None = None,
        *,
        queue: str | None = None,
        priority: int = 0,
    ) -> str:
        return self.enqueue(name, payload, queue=queue, priority=priority)

    async def start_embedded(self, *, ui: bool | None = None) -> Callable[[], Awaitable[None]]:
        if self._stop is None:
            self._stop = await self._worker.start_background()
        if ui if ui is not None else self._ui.enabled:
            self.start_ui()
        return self._stop

    def start_ui(self) -> str:
        if self._ui_server is not None:
            host = self._ui_server.server_address[0]
            port = self._ui_server.server_address[1]
            return f"http://{host}:{port}"

        static_dir = Path(__file__).resolve().parent / "ui_static"
        if not static_dir.is_dir():
            raise RuntimeError(
                f"UI static assets not found at {static_dir}. Run scripts/bundle-ui.sh."
            )

        handler = type(
            "BoundUiHandler",
            (_UiHandler,),
            {"admin_url": self._ui.admin_url},
        )
        server = ThreadingHTTPServer(("127.0.0.1", self._ui.port), handler)
        server.daemon_threads = True

        def serve() -> None:
            os.chdir(static_dir)
            server.serve_forever(poll_interval=0.5)

        thread = threading.Thread(target=serve, name="feather-ui", daemon=True)
        thread.start()
        self._ui_server = server
        self._ui_thread = thread
        url = f"http://127.0.0.1:{self._ui.port}"

        if self._ui.open_browser:
            import webbrowser

            webbrowser.open(url)

        return url

    async def shutdown(self) -> None:
        if self._ui_server:
            self._ui_server.shutdown()
            self._ui_server = None
            self._ui_thread = None
        if self._stop:
            await self._stop()
            self._stop = None
        self._client.close()
