#!/bin/bash
#
# build-rootfs.sh — Build and export the celestia WSL2 rootfs tarball.
#
# Usage:
#   ./build-rootfs.sh                  # build + export to ./dist/
#   ./build-rootfs.sh --tag v3.21.1    # tag with version
#   ./build-rootfs.sh --push           # build + push to registry
#
# Output:
#   dist/celestia-rootfs-<version>.tar.gz   (~15 MB, ready for wsl --import)
#   dist/celestia-rootfs-<version>.sha256   (checksum for integrity verification)
#
# Prerequisites: podman or docker

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TAG="${TAG:-latest}"
DIST_DIR="${SCRIPT_DIR}/dist"

mkdir -p "$DIST_DIR"

# ── Build ─────────────────────────────────────────────────────────────────────

echo "==> Building celestia-wsl2-rootfs:$TAG"
podman build -t "celestia-wsl2-rootfs:$TAG" -f "$SCRIPT_DIR/Dockerfile" "$SCRIPT_DIR"

# ── Export as WSL2-compatible tarball ─────────────────────────────────────────
# WSL2 imports raw root filesystem tarballs (not OCI images). We use
# `podman export` → `gzip` which produces exactly what `wsl --import` expects.
# `podman save` produces an OCI image archive — that's NOT usable for WSL import.

VERSION="${VERSION:-$(date +%Y%m%d)}"
OUTPUT_FILE="$DIST_DIR/celestia-rootfs-${VERSION}.tar.gz"
CHECKSUM_FILE="$DIST_DIR/celestia-rootfs-${VERSION}.sha256"

echo "==> Exporting rootfs tarball (this takes a few seconds)..."
CONTAINER_ID=$(podman create "celestia-wsl2-rootfs:$TAG")
podman export "$CONTAINER_ID" | gzip > "$OUTPUT_FILE"
podman rm "$CONTAINER_ID"

SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)
echo "==> Rootfs tarball: $OUTPUT_FILE ($SIZE)"

# ── Checksum ──────────────────────────────────────────────────────────────────

pushd "$DIST_DIR" > /dev/null
sha256sum "$(basename "$OUTPUT_FILE")" > "$(basename "$CHECKSUM_FILE")"
popd > /dev/null
echo "==> Checksum: $CHECKSUM_FILE"

# ── Done ──────────────────────────────────────────────────────────────────────

echo ""
echo "Distributable artifacts:"
echo "  $OUTPUT_FILE  ($SIZE)"
echo "  $CHECKSUM_FILE"
echo ""
echo "WSL import:"
echo "  wsl --import celestia-000 <install-dir> $OUTPUT_FILE --version 2"
echo ""
echo "Upload to CDN:"
echo "  # Example for GitHub Releases / R2 / OSS"
echo "  gh release upload <tag> $OUTPUT_FILE $CHECKSUM_FILE"
