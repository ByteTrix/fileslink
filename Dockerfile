FROM rust:latest as build

RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY . .

RUN cargo build --release

FROM ubuntu as run

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=build /build/target/release/fileslink /app/fileslink
COPY --from=build /build/target/release/fileslink-cli /app/fileslink-cli

RUN chmod +x /app/fileslink
RUN chmod +x /app/fileslink-cli

ENV PATH="/app:${PATH}"

ENV FILESLINK_PIPE_PATH "/app/fileslink.pipe"
ENV SERVER_PORT ${SERVER_PORT}

EXPOSE ${SERVER_PORT}

CMD ["/app/fileslink"]
