# syntax=docker/dockerfile:1
FROM rust:alpine AS chef
WORKDIR /app
RUN apk add --no-cache build-base
RUN cargo install cargo-chef

FROM chef AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=prepare /app/recipe.json recipe.json
RUN apk add --no-cache openssl-dev
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin gpt-html

FROM alpine AS runtime
ARG COMMIT_SHA
ENV COMMIT_SHA="$COMMIT_SHA"
ENV DOCKER="true"
WORKDIR /app
COPY --from=build /app/target/release/gpt-html .
EXPOSE 9292
CMD ["/app/gpt-html"]
