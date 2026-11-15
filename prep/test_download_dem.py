"""Offline downloader tests: no request reaches NASA or another shared service."""

# Standard-library fixtures and mocks have no extra environment dependencies; C++ test headers.
import hashlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import download_dem


# Isolate file/network behavior from real multi-gigabyte assets; C++ test fixture class.
class DownloadTests(unittest.TestCase):
    # Run the normal download and resumable-prefix paths against an in-memory response; C++ parameterized test.
    def test_download_and_resume(self):
        # Cover empty and partial prefixes without duplicating setup; C++ range-for.
        for offset in [0, 5]:
            # Automatically remove test-only temporary bytes; C++ scoped temporary directory.
            with self.subTest(offset=offset), tempfile.TemporaryDirectory() as name:
                # A known tiny binary payload gives an independent expected digest; C++ byte array.
                payload = b"synthetic DEM bytes"
                asset = {"file": "test.tif", "url": "https://example.invalid/test.tif", "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}
                # Use a path object for the tested binary writer; C++ filesystem::path.
                directory = Path(name)
                # Only the resume fixture begins with saved bytes; C++ conditional prefix write.
                if offset:
                    (directory / "test.tif.part").write_bytes(payload[:offset])
                # BytesIO provides the same context-manager/read protocol as an HTTP response; C++ mock input stream.
                response = io.BytesIO(payload[offset:])
                response.status = 206
                response.headers = {"Content-Range": f"bytes {offset}-{len(payload)-1}/{len(payload)}"}
                # Replace only transport, not the downloader's file or digest logic; C++ injected transport stub.
                with patch("urllib.request.urlopen", return_value=response) as request:
                    download_dem.download(asset, directory)
                    # The requested starting byte must match the existing prefix; C++ mock argument assertion.
                    self.assertEqual(request.call_args.args[0].headers["Range"], f"bytes={offset}-")
                # Final bytes are exact and staging is no longer visible; C++ file-content assertions.
                self.assertEqual((directory / "test.tif").read_bytes(), payload)
                self.assertFalse((directory / "test.tif.part").exists())
                # Reverification of a completed asset must not contact the network; C++ mock expectation.
                with patch("urllib.request.urlopen", side_effect=AssertionError("unexpected HTTP")):
                    download_dem.download(asset, directory, verify_only=True)

    # Missing or altered complete files must fail without a request; C++ error-path test.
    def test_verify_missing_corrupt_and_empty_digest(self):
        # Keep all fixture files isolated; C++ scoped temporary directory.
        with tempfile.TemporaryDirectory() as name:
            # One byte is enough to test size and digest mismatches; C++ descriptor aggregate.
            directory = Path(name)
            asset = {"file": "test.tif", "url": "https://example.invalid/test.tif", "bytes": 1, "sha256": "0" * 64}
            # Verification-only cannot manufacture a missing asset; C++ expected exception.
            with self.assertRaises(FileNotFoundError):
                download_dem.download(asset, directory, verify_only=True)
            # Equal length does not imply equal contents; C++ corrupt-payload fixture.
            (directory / "test.tif").write_bytes(b"x")
            with self.assertRaises(ValueError):
                download_dem.download(asset, directory, verify_only=True)
            # Hashing an empty file is defined independently from downloading an empty DEM; C++ empty-stream digest.
            (directory / "empty").write_bytes(b"")
            self.assertEqual(download_dem.digest(directory / "empty"), hashlib.sha256(b"").hexdigest())

    # A server ignoring Range must not append unrelated bytes; C++ protocol-error test.
    def test_rejects_wrong_range(self):
        # Preserve a known prefix while rejecting the mock response; C++ scoped fixture.
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "test.tif.part").write_bytes(b"a")
            asset = {"file": "test.tif", "url": "https://example.invalid/test.tif", "bytes": 2, "sha256": "0" * 64}
            # A 200 response means the resume contract was not honored; C++ mock status.
            response = io.BytesIO(b"ab")
            response.status = 200
            response.headers = {}
            with patch("urllib.request.urlopen", return_value=response), self.assertRaises(ValueError):
                download_dem.download(asset, directory)
            # The error occurs before opening the append stream; C++ unchanged-content assertion.
            self.assertEqual((directory / "test.tif.part").read_bytes(), b"a")


# Permit direct execution as well as unittest discovery; C++ test runner entry.
if __name__ == "__main__":
    unittest.main()
