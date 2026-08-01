"""Worker lease renewal scheduling."""

from __future__ import annotations

import asyncio
import unittest
from unittest.mock import AsyncMock, MagicMock


class LeaseRenewalTests(unittest.IsolatedAsyncioTestCase):
    async def test_renewal_runs_at_half_lease_interval(self) -> None:
        from getfeather.worker import Worker

        worker = Worker()
        worker._lease_ms = 10_000
        worker._queue_stub = AsyncMock()
        worker._queue_pb2 = MagicMock()
        worker._queue_pb2.ExtendLeaseRequest = MagicMock(side_effect=lambda **kw: kw)

        task = asyncio.create_task(worker._lease_renewal_loop("job-1"))
        await asyncio.sleep(5.1)
        task.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await task

        worker._queue_stub.ExtendLease.assert_awaited()
        call = worker._queue_stub.ExtendLease.await_args
        self.assertEqual(call.args[0]["job_id"], "job-1")
        self.assertEqual(call.args[0]["extension_ms"], 10_000)


if __name__ == "__main__":
    unittest.main()
