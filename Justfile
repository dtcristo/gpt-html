COMMIT_SHA := `git rev-parse HEAD`

dev:
    cargo watch -w src/ -x run

docker-build:
    docker build --build-arg COMMIT_SHA="{{COMMIT_SHA}}" .

docker-build-quiet:
    docker build --build-arg COMMIT_SHA="{{COMMIT_SHA}}" --quiet .

docker-run:
    docker run --env OPENAI_API_KEY --publish 8080:8080 "$(just docker-build-quiet)"

deploy:
    fly deploy --build-arg COMMIT_SHA="{{COMMIT_SHA}}"

logs:
    fly logs
