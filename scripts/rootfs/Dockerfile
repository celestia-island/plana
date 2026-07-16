# celestia WSL2 rootfs — pre-built Alpine with podman.
#
# Build:   podman build -t celestia-wsl2-rootfs .
# Export:  podman save celestia-wsl2-rootfs | gzip > celestia-rootfs.tar.gz
#
# This produces a < 15 MB compressed tarball that `wsl --import` accepts
# directly. Same model as Docker Desktop's self-contained WSL engine:
# lightweight, no host-side effects, zero dependency on user's existing
# WSL distros.
#
# Alpine 3.21 is chosen because:
#   - podman is first-class in Alpine's community repo
#   - minimal attack surface (no systemd, no apt, no snap)
#   - ~8 MB base, ~50 MB installed after podman
#   - same base as Docker Desktop's docker-desktop WSL engine

FROM alpine:3.21

RUN apk add --no-cache \
    podman \
    podman-docker \
    fuse-overlayfs \
    ca-certificates \
    curl \
    bash \
    shadow

RUN mkdir -p /etc/containers \
    && cat > /etc/containers/registries.conf <<'EOF'
[[registry]]
prefix = "docker.io"
location = "docker.io"
EOF

RUN mkdir -p /var/run/podman

VOLUME ["/var/lib/containers"]
CMD ["/bin/sh"]
