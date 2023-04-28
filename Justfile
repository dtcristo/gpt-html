COMMIT_SHA := `git rev-parse HEAD`

dev:
    cargo watch -w src/ -x run

docker-build:
    docker build --build-arg COMMIT_SHA="{{COMMIT_SHA}}" .

docker-build-quiet:
    docker build --build-arg COMMIT_SHA="{{COMMIT_SHA}}" --quiet .

docker-run:
    docker run --rm --env OPENAI_API_KEY --publish 8080:8080 "$(just docker-build-quiet)"

docker-sh:
    docker run --rm --env OPENAI_API_KEY --publish 8080:8080 -it "$(just docker-build-quiet)" sh

deploy:
    fly deploy --build-arg COMMIT_SHA="{{COMMIT_SHA}}"

local PATH:
    http -v --stream --auth "user:${HTTP_BASIC_AUTH_PASSWORD}" localhost:8080/{{PATH}}

remote PATH:
    https -v --stream --auth "user:${HTTP_BASIC_AUTH_PASSWORD}" gpt.dtcristo.com/{{PATH}}
