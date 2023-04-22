FROM rust:1.66.1 as build

WORKDIR /usr/src/esc-api
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian11

COPY --from=build /usr/local/cargo/bin/esc-api /usr/local/bin/esc-api

CMD ["esc-api"]
