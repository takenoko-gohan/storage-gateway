FROM alpine:latest AS prepare

ARG TARGETPLATFORM

COPY target/x86_64-unknown-linux-musl/release/storage-gateway amd64/storage-gateway
COPY target/aarch64-unknown-linux-musl/release/storage-gateway arm64/storage-gateway

RUN echo "Platform: ${TARGETPLATFORM}"
RUN case "${TARGETPLATFORM}" in \
      "linux/amd64") mv amd64/storage-gateway /tmp/storage-gateway ;; \
      "linux/arm64") mv arm64/storage-gateway /tmp/storage-gateway ;; \
      * ) echo "Unsupported platform: ${TARGETPLATFORM}"; exit 1 ;; \
    esac


FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=prepare /tmp/storage-gateway /usr/local/bin/storage-gateway

EXPOSE 8000 8080

ENTRYPOINT ["/usr/local/bin/storage-gateway"]