from __future__ import annotations

import asyncio
import os
import signal
import sys
import uuid
from collections.abc import Awaitable, Callable
from pathlib import Path
from typing import Any

import grpc

from feather.client import _import_queue_stubs

Handler = Callable[["JobContext"], Awaitable[None]]


class JobContext:
    def __init__(
        self,
        *,
        job_id: str,
        queue: str,
        name: str,
        payload: bytes,
        ack: Callable[[], Awaitable[None]],
        nack: Callable[[str], Awaitable[None]],
    ) -> None:
        self.id = job_id
        self.queue = queue
        self.name = name
        self.payload = payload
        self.ack = ack
        self.nack = nack


class Worker:
    def __init__(
        self,
        *,
        address: str | None = None,
        worker_id: str | None = None,
        queues: list[str] | None = None,
    ) -> None:
        self.address = address or os.environ.get("FEATHER_ADDRESS", "localhost:50051")
        self.worker_id = worker_id or f"python-{uuid.uuid4().hex[:8]}"
        self.queues = queues or ["default"]
        self._handlers: dict[str, Handler] = {}
        self._running = False
        self._lease_ms = 30_000
        self._heartbeat_ms = 10_000
        self._embedded_task: asyncio.Task[None] | None = None
        self._hb_task: asyncio.Task[None] | None = None
        self._channel: grpc.aio.Channel | None = None
        self._worker_stub: Any = None
        self._queue_stub: Any = None
        self._queue_pb2: Any = None

    def task(self, name: str, handler: Handler) -> Worker:
        self._handlers[name] = handler
        return self

    async def _setup(self) -> None:
        queue_pb2, queue_pb2_grpc = _import_queue_stubs()
        self._queue_pb2 = queue_pb2
        _gen_dir = Path(__file__).resolve().parent / "_gen"
        if str(_gen_dir) not in sys.path:
            sys.path.insert(0, str(_gen_dir))
        from feather.v1 import worker_pb2, worker_pb2_grpc  # type: ignore

        self._channel = grpc.aio.insecure_channel(self.address)
        self._queue_stub = queue_pb2_grpc.QueueServiceStub(self._channel)
        self._worker_stub = worker_pb2_grpc.WorkerServiceStub(self._channel)

        reg = await self._worker_stub.Register(
            worker_pb2.RegisterRequest(worker_id=self.worker_id, queues=self.queues)
        )
        self._lease_ms = reg.lease_duration_ms or 30_000
        self._heartbeat_ms = reg.heartbeat_interval_ms or 10_000

    async def _heartbeat_loop(self) -> None:
        from feather.v1 import worker_pb2  # type: ignore

        while self._running:
            try:
                await self._worker_stub.Heartbeat(
                    worker_pb2.HeartbeatRequest(worker_id=self.worker_id)
                )
            except Exception:
                pass
            await asyncio.sleep(self._heartbeat_ms / 1000)

    async def _poll_loop(self) -> None:
        assert self._queue_stub is not None and self._queue_pb2 is not None
        while self._running:
            res = await self._queue_stub.Dequeue(
                self._queue_pb2.DequeueRequest(
                    worker_id=self.worker_id,
                    queues=self.queues,
                    wait_timeout_ms=30_000,
                )
            )
            if not res.job.id:
                await asyncio.sleep((res.backoff_hint_ms or 500) / 1000)
                continue
            await self._handle_job(res.job)

    async def _handle_job(self, job: Any) -> None:
        assert self._queue_stub is not None and self._queue_pb2 is not None

        async def ack() -> None:
            await self._queue_stub.Ack(
                self._queue_pb2.AckRequest(job_id=job.id, worker_id=self.worker_id)
            )

        async def nack(reason: str = "error") -> None:
            await self._queue_stub.Nack(
                self._queue_pb2.NackRequest(
                    job_id=job.id, worker_id=self.worker_id, reason=reason
                )
            )

        ctx = JobContext(
            job_id=job.id,
            queue=job.queue,
            name=job.name,
            payload=bytes(job.payload),
            ack=ack,
            nack=nack,
        )
        handler = self._handlers.get(job.name)
        if not handler:
            await nack(f"no handler for {job.name}")
            return
        try:
            await handler(ctx)
            await ack()
        except Exception as exc:
            await nack(str(exc))

    async def start(self) -> None:
        """Block until stopped — dedicated worker process."""
        await self._setup()
        self._running = True
        loop = asyncio.get_running_loop()

        def _stop(*_: Any) -> None:
            self._running = False

        for sig in (signal.SIGINT, signal.SIGTERM):
            try:
                loop.add_signal_handler(sig, _stop)
            except NotImplementedError:
                pass

        self._hb_task = asyncio.create_task(self._heartbeat_loop())
        try:
            await self._poll_loop()
        finally:
            await self._teardown()

    async def start_background(self) -> Callable[[], Awaitable[None]]:
        """Celery-style embedded mode — poll without blocking the event loop."""
        await self._setup()
        self._running = True
        self._hb_task = asyncio.create_task(self._heartbeat_loop())
        self._embedded_task = asyncio.create_task(self._poll_loop())

        async def stop() -> None:
            await self._teardown()

        return stop

    async def _teardown(self) -> None:
        self._running = False
        if self._hb_task:
            self._hb_task.cancel()
        if self._embedded_task:
            self._embedded_task.cancel()
        if self._worker_stub and self._channel:
            try:
                from feather.v1 import worker_pb2  # type: ignore

                await self._worker_stub.Deregister(
                    worker_pb2.DeregisterRequest(worker_id=self.worker_id)
                )
            except Exception:
                pass
            await self._channel.close()
