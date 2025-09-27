#include <math.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static inline double rad(double rad) { return rad*256.0/(2*M_PI); }
static inline double pos(double pos) { return pos*256.0; }

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

  double state[7] = {0.0, 13.0, -300.0, 0.0, 0.0, 0.0, 0.0};

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

    time_t t = time(NULL);
    struct tm jtime = *gmtime(&t);
    jtime.tm_hour = (jtime.tm_hour + 9)%24;

    double theta =
        ((double)jtime.tm_hour + jtime.tm_min/60.0)/24.0*2*M_PI;

    state[4] = sin(theta);
    state[5] = -cos(theta);
    state[6] = 0;

    if (type == 'v') {
      double pos_x_r = pos(state[0]);
      double pos_y_r = pos(state[1]);
      double pos_z_r = pos(state[2]);
      double yaw_r = rad(state[3]);
      double sun_x_r = pos(state[4]);
      double sun_y_r = pos(state[5]);
      double sun_z_r = pos(state[6]);

      char cmd[512];
      snprintf(cmd, sizeof(cmd),
               "./asahi_renderer/render %f %f %f %f %f %f %f | ffmpeg "
               "-loglevel 0 -i - -f gif -",
               pos_x_r, pos_y_r, pos_z_r, yaw_r, sun_x_r, sun_y_r, sun_z_r);

      FILE *fp = popen(cmd, "r");
      if (!fp) {
        close(sock);
        continue;
      }

      unsigned char *gif_data = NULL;
      size_t gif_size = 0;
      size_t cap = 8192;
      gif_data = malloc(cap);
      if (!gif_data) {
        pclose(fp);
        close(sock);
        continue;
      }

      size_t r;
      while ((r = fread(gif_data + gif_size, 1, cap - gif_size, fp)) > 0) {
        gif_size += r;
        if (gif_size == cap) {
          cap *= 2;
          unsigned char *tmp = realloc(gif_data, cap);
          if (!tmp) {
            free(gif_data);
            pclose(fp);
            close(sock);
            continue;
          }
          gif_data = tmp;
        }
      }
      pclose(fp);

      char header[256];
      int hlen = snprintf(header, sizeof(header),
                          "HTTP/1.1 200 OK\r\n"
                          "Cache-Control: max-age=0\r\n"
                          "Connection: close\r\n"
                          "Content-Type: image/gif\r\n"
                          "Content-Length: %zu\r\n\r\n",
                          gif_size);
      send(sock, header, hlen, 0);
      send(sock, gif_data, gif_size, 0);

      free(gif_data);
    }
    else {
      if (type == 'd')
        state[3] -= M_PI/4.0;
      else if (type == 'a')
        state[3] += M_PI/4.0;
      else if (type == 'w') {
        state[0] -= state[1]*sin(state[3]);
        state[2] -= state[1]*cos(state[3]);
      }
      else if (type == 's')
        state[3] += M_PI;

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