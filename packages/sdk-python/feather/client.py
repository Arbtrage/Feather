from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Any

import grpc

_PROTO_DIR = Path(__file__).resolve().parent / "proto"
_GEN_DIR = Path(__file__).resolve().parent / "_gen"


def _ensure_generated() -> None:
    if (_GEN_DIR / "feather" / "v1" / "queue_pb2_grpc.py").exists():
        return
    from grpc_tools import protoc

    _GEN_DIR.mkdir(parents=True, exist_ok=True)
    proto_files = [
        str(_PROTO_DIR / "feather/v1/common.proto"),
        str(_PROTO_DIR / "feather/v1/job.proto"),
        str(_PROTO_DIR / "feather/v1/queue.proto"),
        str(_PROTO_DIR / "feather/v1/worker.proto"),
    ]
    args = [
        "grpc_tools.protoc",
        f"-I{str(_PROTO_DIR)}",
        f"--python_out={str(_GEN_DIR)}",
        f"--grpc_python_out={str(_GEN_DIR)}",
        *proto_files,
    ]
    if protoc.main(args) != 0:
        raise RuntimeError("failed to generate python protos")


def _import_queue_stubs():
    _ensure_generated()
    if str(_GEN_DIR) not in sys.path:
        sys.path.insert(0, str(_GEN_DIR))
    from feather.v1 import queue_pb2, queue_pb2_grpc  # type: ignore

    return queue_pb2, queue_pb2_grpc


class FeatherClient:
    def __init__(self, address: str | None = None) -> None:
        self.address = address or os.environ.get("FEATHER_ADDRESS", "localhost:50051")
        queue_pb2, queue_pb2_grpc = _import_queue_stubs()
        self._queue_pb2 = queue_pb2
        self._channel = grpc.insecure_channel(self.address)
        self._stub = queue_pb2_grpc.QueueServiceStub(self._channel)

    def enqueue(
        self,
        name: str,
        *,
        queue: str = "default",
        payload: bytes = b"",
        priority: int = 0,
    ) -> str:
        req = self._queue_pb2.EnqueueRequest(
            queue=queue, name=name, payload=payload, priority=priority
        )
        res = self._stub.Enqueue(req)
        return res.job_id

    def get_job(self, job_id: str) -> dict[str, Any] | None:
        res = self._stub.GetJob(self._queue_pb2.GetJobRequest(job_id=job_id))
        if not res.job.id:
            return None
        j = res.job
        return {
            "id": j.id,
            "queue": j.queue,
            "name": j.name,
            "payload": bytes(j.payload),
            "state": j.state,
        }

    def close(self) -> None:
        self._channel.close()
