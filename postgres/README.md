docker build -t emotes-postgres

docker run -d --name twitch-emotes-db -e POSTGRES_USER=kmw -e POSTGRES_PASSWORD=bald emotes-postgres

psql -U kmw