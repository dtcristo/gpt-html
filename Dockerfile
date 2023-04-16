# syntax = docker/dockerfile:1

FROM ruby:3.2-alpine

WORKDIR /app

RUN apk add build-base

# Install application gems
COPY Gemfile Gemfile.lock ./
RUN bundle install

# Copy application code
COPY . .

# Start the server by default, this can be overwritten at runtime
EXPOSE 9292
CMD ["puma"]
