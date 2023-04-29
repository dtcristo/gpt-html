# syntax=docker/dockerfile:1
FROM --platform=linux/amd64 clux/muslrust:stable AS chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=prepare /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin gpt-html

FROM --platform=linux/amd64 gcr.io/distroless/static AS runtime
WORKDIR /app
ARG COMMIT_SHA
ENV COMMIT_SHA="$COMMIT_SHA" \
    DOCKER="true"
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/gpt-html .
EXPOSE 9292
CMD ["./gpt-html"]
