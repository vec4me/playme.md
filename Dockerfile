FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y tcc ffmpeg git && rm -rf /var/lib/apt/lists/*

WORKDIR /src

COPY . .

RUN git clone https://github.com/vec4me/asahi_renderer.git asahi_renderer

RUN cd asahi_renderer && sh build.sh && cd ..

RUN tcc -O2 -o serve serve.c

ENV PORT=8080
EXPOSE 8080

CMD ["sh", "-c", "./serve $PORT"]
