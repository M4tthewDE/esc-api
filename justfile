docker-build:
    docker build --tag esc-api:latest .

docker-run:
    docker run -d -p 8080:8080 --name esc-api esc-api:latest
