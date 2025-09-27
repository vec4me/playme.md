#include <math.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

// Degrees → radians
static inline double deg2rad(double deg) { return deg*M_PI/180.0; }

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Usage: %s <port>\n", argv[0]);
    return 1;
  }

  int port = atoi(argv[1]);
  int server = socket(AF_INET, SOCK_STREAM, 0);
  if (server < 0) {
    perror("socket");
    return 1;
  }

  struct sockaddr_in addr = {0};
  addr.sin_family = AF_INET;
  addr.sin_port = htons(port);
  addr.sin_addr.s_addr = INADDR_ANY;

  if (bind(server, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
    perror("bind");
    return 1;
  }
  if (listen(server, 5) < 0) {
    perror("listen");
    return 1;
  }

  double state[7] = {0, 3600, -100000, 0, 0, 0, 0};

  while (1) {
    struct sockaddr_in client;
    socklen_t len = sizeof(client);
    int sock = accept(server, (struct sockaddr *)&client, &len);
    if (sock < 0)
      continue;

    char buffer[1024];
    ssize_t n = recv(sock, buffer, sizeof(buffer) - 1, 0);
    if (n <= 0) {
      close(sock);
      continue;
    }
    buffer[n] = 0;

    char type = (strncmp(buffer, "GET /", 5) == 0) ? buffer[5] : 0;

   //Japan time sun update
    time_t t = time(NULL);
    struct tm jtime = *gmtime(&t);
    jtime.tm_hour = (jtime.tm_hour + 9)%24;
    double sun_angle =
        ((jtime.tm_hour*60 + jtime.tm_min)*360.0)/(24*60);
    state[4] = sin(deg2rad(sun_angle + 90))*127;
    state[5] = sin(deg2rad(sun_angle - 90))*127;
    state[6] = 0;

    if (type == 'v') {
     //Use ffmpeg pipe for GIF, let it manage size
      char cmd[512];
      snprintf(cmd, sizeof(cmd),
               "./asahi_renderer/render %f %f %f %f %f %f %f | "
               "ffmpeg -loglevel 0 -i - -f gif -",
               state[0], state[1], state[2], state[3], state[4], state[5],
               state[6]);

      FILE *fp = popen(cmd, "r");
      if (!fp) {
        close(sock);
        continue;
      }

     //Send HTTP header first
      const char *header = "HTTP/1.1 200 OK\r\n"
                           "Cache-Control: max-age=0\r\n"
                           "Connection: close\r\n"
                           "Content-Type: image/gif\r\n\r\n";
      send(sock, header, strlen(header), 0);

     //Stream GIF directly to socket
      char buf[8192];
      size_t r;
      while ((r = fread(buf, 1, sizeof(buf), fp)) > 0) {
        send(sock, buf, r, 0);
      }
      pclose(fp);
    }
    else {
      if (type == 'r') {
        state[3] -= 32;//rotate right
      }
      else if (type == 'l') {
        state[3] += 32;//rotate left
      }
      else if (type == 'u') {
        state[0] -= 30*sin(deg2rad(state[3]));
        state[2] -= 30*sin(deg2rad(state[3] + 90));
      }
      else if (type == 'd') {
        state[3] += 64;//down
      }

      const char *redirect = "HTTP/1.1 302 Found\r\n"
                             "Connection: close\r\n"
                             "Location: https://github.com/vec4me\r\n\r\n";
      send(sock, redirect, strlen(redirect), 0);
    }

    close(sock);
  }

  close(server);
  return 0;
}