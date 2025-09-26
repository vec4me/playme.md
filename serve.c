#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <netinet/in.h>

#define endof(a) sizeof a - 1

static int sin[] = {0, 3, 6, 9, 12, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46, 49, 51, 54, 57, 60, 63, 65, 68, 71, 73, 76, 78, 81, 
 83, 85, 88, 90, 92, 94, 96, 98, 100, 102, 104, 106, 107, 109, 111, 112, 113, 115, 116, 117, 118, 120, 121, 122, 
 122, 123, 124, 125, 125, 126, 126, 126, 127, 127, 127, 127, 127, 127, 127, 126, 126, 126, 125, 125, 124, 123, 
 122, 122, 121, 120, 118, 117, 116, 115, 113, 112, 111, 109, 107, 106, 104, 102, 100, 98, 96, 94, 92, 90, 88, 
 85, 83, 81, 78, 76, 73, 71, 68, 65, 63, 60, 57, 54, 51, 49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 12, 9, 
 6, 3, 0, -3, -6, -9, -12, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46, -49, -51, -54, -57, -60, -63, 
 -65, -68, -71, -73, -76, -78, -81, -83, -85, -88, -90, -92, -94, -96, -98, -100, -102, -104, -106, -107, -109, 
 -111, -112, -113, -115, -116, -117, -118, -120, -121, -122, -122, -123, -124, -125, -125, -126, -126, -126, 
 -127, -127, -127, -127, -127, -127, -127, -126, -126, -126, -125, -125, -124, -123, -122, -122, -121, -120, 
 -118, -117, -116, -115, -113, -112, -111, -109, -107, -106, -104, -102, -100, -98, -96, -94, -92, -90, -88, 
 -85, -83, -81, -78, -76, -73, -71, -68, -65, -63, -60, -57, -54, -51, -49, -46, -43, -40, -37, -34, -31, -28, 
 -25, -22, -19, -16, -12, -9, -6, -3};

int main(int argc, char *argv[]) {
	if (argc < 2) {
		fprintf(stderr, "Usage: %s <port>\n", argv[0]);
		return 1;
	}

	int port = atoi(argv[1]);

	printf("%i\n", port);

	int cos[sizeof sin];
	for (int i = sizeof sin; i--;) cos[i] = sin[(i + 65)&255];

	struct sockaddr_in info_u;
	info_u.sin_family = AF_INET;
	info_u.sin_port = htons(port);
	info_u.sin_addr.s_addr = INADDR_ANY;

	int sock_u = socket(AF_INET, SOCK_STREAM, 0);
	bind(sock_u, (struct sockaddr *)&info_u, sizeof info_u);
	listen(sock_u, 0);

	struct sockaddr_in info_t;
	unsigned info_t_s = sizeof info_t;

	enum { cx, cy, cz, ry, sdx, sdy, sdz, n_acts };
	int arg[] = { 0, 10000, 0, 0, 0, 0, 127 };

	char buff_i[6]; // The request should only be 3 letters long (GET) or this will break completely
	char buff_o[400000] = "HTTP/1.1 200 OK\r\nCache-Control: max-age=0\r\nConnection: close\r\nContent-Type: image/gif\r\n\r\n";
	size_t head_s = 89;

	char cmd[255];

	for (;;) {
		int sock_t = accept(sock_u, (struct sockaddr *)&info_t, &info_t_s);
		recv(sock_t, buff_i, sizeof buff_i, 0);
		char type = buff_i[5];

		putchar(type);
		putchar('\n');

		if (type == 'v') {
			sprintf(cmd, "./asahi_renderer/render %f %f %f %f %f %f %f | ffmpeg -loglevel 0 -i - -f gif -vf 'split[a][b];[a]palettegen[p];[b][p]paletteuse' -", (double)arg[cx], (double)arg[cy], (double)arg[cz], (double)arg[ry], (double)arg[sdx], (double)arg[sdy], (double)arg[sdz]);

			FILE *hand = popen(cmd, "r");
			fread(buff_o + head_s, sizeof buff_o - head_s, 1, hand);
			pclose(hand);

			char *term = buff_o + endof(buff_o);
			while (*--term != ';');

			send(sock_t, buff_o, 1 + (size_t)(term - buff_o), 0);
		}
		else {
			if (type == 'r') arg[ry] -= 32;
			if (type == 'l') arg[ry] += 32;
			if (type == 'u') {
				arg[cx] -= 33*sin[arg[ry]&255];
				arg[cz] += 33*cos[arg[ry]&255];
			}
			if (type == 'd') arg[ry] += 64;

			send(sock_t, "HTTP/1.1 302 Found\r\nConnection: close\r\nLocation: https://github.com/vec4me\r\n\r\nWHAT THE FUCK", 94, 0);
		}

		close(sock_t);
	}
}
