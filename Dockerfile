FROM rust:1.70 as build

WORKDIR /usr/src/esc-api
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian11

COPY --from=build /usr/local/cargo/bin/esc-api /usr/local/bin/esc-api
COPY --from=build /usr/src/esc-api/config.toml /config.toml
COPY --from=build /usr/src/esc-api/countries.json /countries.json

CMD ["esc-api"]
