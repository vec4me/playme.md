FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y tcc ffmpeg

WORKDIR /src
COPY . .

RUN tcc -O2 -o serve serve.c

ENV PORT=8080
EXPOSE 8080

CMD ["./serve"]