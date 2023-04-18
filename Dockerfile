FROM ruby:3.2-alpine as base
WORKDIR /app

FROM base as build
COPY Gemfile Gemfile.lock .
RUN set -eux; \
    apk add --no-cache build-base; \
    bundle install;

FROM base as app
COPY --from=build /usr/local/bundle /usr/local/bundle
COPY . .
EXPOSE 9292
CMD ["puma"]
