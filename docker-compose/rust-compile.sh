mv docker-compose.override.yml.rust docker-compose.override.yml
docker compose run --rm rust-builder
mv docker-compose.override.yml docker-compose.override.yml.rust