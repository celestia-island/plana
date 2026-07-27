#!/bin/bash
#
# build-images.sh — Pre-pull and bundle container images for offline install.
#
# Usage:
#   ./build-images.sh                    # pull + export all images
#   ./build-images.sh --platform linux/amd64  # cross-platform pull
#   ./build-images.sh --offline <dir>    # generate an offline bundle ready for USB/CDN
#
# Output:
#   dist/celestia-images-<version>.tar.gz   (pre-pulled container images)
#   dist/celestia-images-<version>.sha256   (checksum)
#
# At install time, the bundled images are loaded with:
#   podman load -i celestia-images-<version>.tar.gz
#
# This eliminates network dependency for the three core container images
# (postgres, pgvector, registry), saving ~300MB of flaky Docker Hub pulls
# during initial setup.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="${SCRIPT_DIR}/dist"
PLATFORM="${PLATFORM:-linux/amd64}"

mkdir -p "$DIST_DIR"

# ── Image manifest ────────────────────────────────────────────────────────────
# These are the only images needed for a minimal entelecheia deployment.
# entelecheia:latest is NOT included — it must be built from source or
# distributed as a separate pre-built OCI image (see build-scepter.sh).
# The scepter binary depends on the user's LLM API key in .env, so it
# cannot be fully pre-configured.

IMAGES=(
    "docker.io/library/postgres:16-alpine"
    "docker.io/pgvector/pgvector:pg18-bookworm"
    "docker.io/library/registry:2"
)

# ── Pull ──────────────────────────────────────────────────────────────────────

echo "==> Pulling container images (platform: $PLATFORM)"
PULLED=()
for img in "${IMAGES[@]}"; do
    echo "  Pulling $img ..."
    podman pull --platform "$PLATFORM" "$img"
    PULLED+=("$img")
done
echo "==> All images pulled successfully"

# ── Export as single tarball ──────────────────────────────────────────────────

VERSION="${VERSION:-$(date +%Y%m%d)}"
OUTPUT_FILE="$DIST_DIR/celestia-images-${VERSION}.tar.gz"
CHECKSUM_FILE="$DIST_DIR/celestia-images-${VERSION}.sha256"

echo "==> Exporting images to single tarball..."
# podman save supports multiple images in one archive.
# We pipe through gzip for smaller transfer size.
podman save "${PULLED[@]}" | gzip > "$OUTPUT_FILE"

SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)

# ── Checksum ──────────────────────────────────────────────────────────────────

pushd "$DIST_DIR" > /dev/null
sha256sum "$(basename "$OUTPUT_FILE")" > "$(basename "$CHECKSUM_FILE")"
popd > /dev/null

# ── Done ──────────────────────────────────────────────────────────────────────

echo ""
echo "Distributable artifacts:"
echo "  $OUTPUT_FILE  ($SIZE)"
echo "  $CHECKSUM_FILE"
echo ""
echo "Install-time load:"
echo "  podman load -i $OUTPUT_FILE"
echo ""
echo "Contained images:"
for img in "${PULLED[@]}"; do
    echo "  $img"
done
