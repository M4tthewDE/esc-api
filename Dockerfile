FROM rust:1.82 as build

WORKDIR /usr/src/esc-api

COPY src/ src/
COPY Cargo.toml .
COPY Cargo.lock .
COPY countries.json .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian12

COPY --from=build /usr/local/cargo/bin/esc-api /usr/local/bin/esc-api
COPY --from=build /usr/src/esc-api/countries.json /countries.json

CMD ["esc-api"]
