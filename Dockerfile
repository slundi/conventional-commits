# Distroless multi-arch image for commit-check.
#
# Built from pre-compiled MUSL binaries (not from source), so the build is
# fast and the image stays ~10 MB. Expects the build context to contain
# `commit-check-linux-$TARGETARCH` (amd64, arm64, arm) next to this file —
# populate them with `just docker-context VERSION`, which takes them from
# dist/ or the GitHub release.
#
# There is no RUN step, so foreign platforms build without QEMU emulation.
#
# Build locally for the host platform:
#     just docker-context v0.2.0
#     podman build -t commit-check:dev .
#
# Build multi-arch and push to Docker Hub:
#     just docker-publish v0.2.0

FROM gcr.io/distroless/static-debian12:nonroot

ARG TARGETARCH
COPY commit-check-linux-${TARGETARCH} /usr/local/bin/commit-check

USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/commit-check"]
