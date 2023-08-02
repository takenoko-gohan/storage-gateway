FROM alpine:latest AS prepare

ARG TARGETPLATFORM

COPY target/x86_64-unknown-linux-gnu/release/s3-gateway amd64/s3-gateway
COPY target/aarch64-unknown-linux-gnu/release/s3-gateway arm64/s3-gateway

RUN echo "Platform: ${TARGETPLATFORM}"
RUN case "${TARGETPLATFORM}" in \
      "linux/amd64") mv amd64/s3-gateway /tmp/s3-gateway ;; \
      "linux/arm64") mv arm64/s3-gateway /tmp/s3-gateway ;; \
      * ) echo "Unsupported platform: ${TARGETPLATFORM}"; exit 1 ;; \
    esac


FROM gcr.io/distroless/cc-debian11

COPY --from=prepare /tmp/s3-gateway /usr/local/bin/s3-gateway

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/s3-gateway"]