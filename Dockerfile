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
ENV COMMIT_SHA=$COMMIT_SHA
WORKDIR /app
COPY --from=build /app/target/release/gpt-html .
EXPOSE 9292
CMD ["/app/gpt-html"]
