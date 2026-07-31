from __future__ import annotations

import json
from collections.abc import Awaitable, Callable
from typing import Any

from feather.client import FeatherClient
from feather.worker import Handler, Worker

EnqueuePayload = bytes | str | dict[str, Any]


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
    ) -> None:
        self._client = FeatherClient(address)
        self._default_queue = queue
        self._worker = Worker(
            address=address,
            worker_id=worker_id,
            queues=queues or [queue],
        )
        self._stop: Callable[[], Awaitable[None]] | None = None

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

    async def start_embedded(self) -> Callable[[], Awaitable[None]]:
        if self._stop is None:
            self._stop = await self._worker.start_background()
        return self._stop

    async def shutdown(self) -> None:
        if self._stop:
            await self._stop()
            self._stop = None
        self._client.close()
