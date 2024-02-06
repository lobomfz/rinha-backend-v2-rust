FROM rustlang/rust:nightly as base
WORKDIR /usr/src/app

FROM base AS build
RUN mkdir -p /temp/prod
COPY Cargo.lock /temp/prod/
COPY Cargo.toml /temp/prod/
COPY src /temp/prod/src
COPY .sqlx /temp/prod/.sqlx
RUN cd /temp/prod && cargo build -j3 --release

FROM base AS release
COPY --from=build /temp/prod/target/release/rinha-rust rinha-rust

ENTRYPOINT [ "./rinha-rust" ]
