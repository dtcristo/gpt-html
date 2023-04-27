deploy:
    fly deploy --env COMMIT_SHA="$(git rev-parse HEAD)"
