docker-build:
    docker build --tag esc-api:latest .

docker-run:
    docker run -d -p 8080:8080 --name esc-api esc-api:latest

docker-run-release:
    docker run -d -p 8080:8080 --name esc-api gcr.io/esc-api-384517/esc-api

docker-clean:
    docker container stop esc-api
    docker container rm esc-api

docker-build-release:
    docker build --tag gcr.io/esc-api-384517/esc-api:latest .

docker-push-release:
    docker push gcr.io/esc-api-384517/esc-api:latest

post-ranking:
    curl -X POST localhost:8080/ranking \
    -H 'Content-Type: application/json' \
    -H 'Authorization: hunter2' \
    -d '{"name":"test", "countries": ["Germany", "England", "France", "Italy"]}'

get-ranking NAME:
    curl -X GET localhost:8080/ranking/{{NAME}} -H 'Authorization: hunter2'

get-result:
    curl -v localhost:8080/result -H 'Authorization: hunter2'
