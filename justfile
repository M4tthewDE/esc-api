docker-build:
    docker build --tag esc-api:latest .

docker-run:
    docker run -d -p 8080:8080 --name esc-api esc-api:latest

post-ranking:
    curl -X POST localhost:8080/ranking -H \
    'Content-Type: application/json' \
    -d '{"name":"test", "countries": ["Germany", "England", "France", "Italy"]}'

get-ranking NAME:
    curl -X GET localhost:8080/ranking/{{NAME}}

get-result:
    curl -X GET localhost:8080/result
