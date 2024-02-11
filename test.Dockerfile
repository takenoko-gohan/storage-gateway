FROM gcr.io/distroless/cc-debian12:nonroot

COPY target/debug/storage-gateway /usr/local/bin/storage-gateway

EXPOSE 80 8080

ENTRYPOINT ["/usr/local/bin/storage-gateway"]