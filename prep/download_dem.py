#!/usr/bin/env python3
"""Download only the fixed-site manifest, outside the repository, and verify SHA-256."""

# Native standard-library modules avoid an additional package environment; C++ standard headers.
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import urllib.request


# Hash large rasters with bounded RAM; C++ function reading a buffered ifstream.
def digest(path):
    # SHA-256 identifies the published bytes, not just a similarly sized TIFF; C++ digest context.
    value = hashlib.sha256()
    # A context manager closes the file on success and exceptions; C++ RAII ifstream.
    with path.open("rb") as source:
        # Stream 8 MiB blocks rather than allocating gigabytes; C++ while(read(...)).
        while block := source.read(8 * 1024 * 1024):
            # Feed exactly the downloaded bytes into the digest; C++ hash.update.
            value.update(block)
    # Hexadecimal text is directly comparable to the manifest; C++ digest-to-hex utility.
    return value.hexdigest()


# Support resumable downloads without exposing an incomplete final filename; C++ asset-download function.
def download(asset, directory, verify_only=False):
    # Join a trusted manifest basename under the explicitly selected data directory; C++ filesystem::path.
    target = directory / asset["file"]
    # A final filename must already match both size and checksum; C++ existence/verification branch.
    if target.exists():
        # Do not silently overwrite a corrupt or different published version; C++ checked precondition.
        if target.stat().st_size != asset["bytes"] or digest(target) != asset["sha256"]:
            raise ValueError(f"Existing DEM does not match manifest: {target}")
    # Verification-only mode performs no network I/O; C++ read-only operation branch.
    elif verify_only:
        raise FileNotFoundError(target)
    else:
        # Incomplete bytes remain resumable in a visibly separate file; C++ staging path.
        partial = target.with_name(target.name + ".part")
        # File length is the contiguous successfully written prefix; C++ filesystem::file_size.
        offset = partial.stat().st_size if partial.exists() else 0
        # A too-large prefix cannot be a valid resume of this asset; C++ bounds guard.
        if offset > asset["bytes"]:
            raise ValueError(f"Oversized partial DEM: {partial}")
        # An already complete prefix needs only checksum verification; C++ conditional HTTP operation.
        if offset < asset["bytes"]:
            # Range resumes explicitly require the correct response offset; C++ request headers.
            request = urllib.request.Request(asset["url"], headers={"Range": f"bytes={offset}-"})
            # Bound socket inactivity and append only after validating the response; C++ RAII response.
            with urllib.request.urlopen(request, timeout=180) as response:
                # Reject caches/servers ignoring a resume request; C++ status/header validation.
                if response.status != 206 or not response.headers.get("Content-Range", "").startswith(f"bytes {offset}-"):
                    raise ValueError(f"Server did not honor byte range: {asset['url']}")
                # Append exact response bytes without decompressing/re-encoding the TIFF; C++ binary append stream.
                with partial.open("ab") as output:
                    # Limit transient memory per worker; C++ buffered-copy loop.
                    while block := response.read(8 * 1024 * 1024):
                        output.write(block)
        # Verify before publishing the final path; C++ size and SHA-256 preconditions.
        if partial.stat().st_size != asset["bytes"] or digest(partial) != asset["sha256"]:
            raise ValueError(f"Downloaded DEM does not match manifest: {partial}")
        # Rename on the same filesystem publishes a completed file atomically; C++ filesystem::rename.
        partial.rename(target)
    # Emit an auditable per-file result; C++ formatted stream output.
    print(f"Verified {asset['file']}: {asset['bytes']:,} bytes, SHA-256 {asset['sha256']}", flush=True)


# Parse the small command interface before any network request; C++ program entry function.
def main():
    # The output directory is explicit to avoid accidentally vendoring DEMs; C++ CLI parser.
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    arguments = parser.parse_args()
    # Read the versioned manifest next to this script, not a remote mutable file; C++ JSON file parsing.
    assets = json.loads(Path(__file__).with_name("dem-manifest.json").read_text())
    # Validate all paths, sizes and digests before creating files; C++ all_of predicate.
    if not assets or any(Path(a["file"]).name != a["file"] or a["bytes"] <= 0 or len(a["sha256"]) != 64 for a in assets):
        raise ValueError("Invalid DEM manifest")
    # Enforce the user's approximate 20 GB ceiling before starting downloads; C++ total-size reduction.
    total = sum(a["bytes"] for a in assets)
    if total > 20_000_000_000:
        raise ValueError(f"{total:,} bytes exceeds the 20 GB download gate")
    # Resolve symbolic paths before rejecting any directory inside the repository; C++ canonical paths.
    directory = arguments.directory.expanduser().resolve()
    repository = Path(__file__).resolve().parents[1]
    if directory == repository or repository in directory.parents:
        raise ValueError("DEM directory must be outside the repository")
    # Create only the caller's explicit external directory; C++ create_directories.
    directory.mkdir(parents=True, exist_ok=True)
    # Announce total disk usage before requests; C++ formatted diagnostic.
    print(f"Manifest: {len(assets)} DEMs, {total:,} bytes; directory={directory}", flush=True)
    # Four concurrent files are enough without issuing many requests to the public archive; C++ bounded worker pool.
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        # Materialization propagates every worker's failure to the command exit status; C++ join/get futures.
        list(pool.map(lambda asset: download(asset, directory, arguments.verify_only), assets))


# Importing the downloader for tests performs no network work; C++ main separated from library helpers.
if __name__ == "__main__":
    main()
