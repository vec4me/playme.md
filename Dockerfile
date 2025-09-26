FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y tcc ffmpeg git && rm -rf /var/lib/apt/lists/*

WORKDIR /src

COPY . .

RUN git submodule update --init --recursive

RUN tcc -O2 -o serve serve.c

ENV PORT=8080

EXPOSE $PORT

CMD ["sh", "-c", "./serve $PORT"]