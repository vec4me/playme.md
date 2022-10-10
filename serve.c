#include <stdio.h>
#include <unistd.h>
#include <netinet/in.h>
#include <math.h>

int main() {
	struct sockaddr_in info_u;
	info_u.sin_family = AF_INET;
	info_u.sin_port = htons(7890);
	info_u.sin_addr.s_addr = INADDR_ANY;

	int32_t sock_u = socket(AF_INET, SOCK_STREAM, 0);
	bind(sock_u, (struct sockaddr *)&info_u, sizeof info_u);
	listen(sock_u, 0);

	struct sockaddr_in info_t;
	uint32_t info_t_s = sizeof info_t;

	float cx = 2000;
	float cz = 2000;

	float ry = 0;

	float sdx = 1;
	float sdy = 0;
	float sdz = 0;

	char buff_i[6]; // The request should only be 3 letters long (GET) or this will break completely
	char buff_o[30000] = "HTTP/1.1 200 OK\r\nCache-Control: max-age=0\r\nConnection: close\r\nContent-Type: image/gif\r\n\r\n";
	size_t head_l = 89;

	for (;;) {
		int32_t sock_t = accept(sock_u, (struct sockaddr *)&info_t, &info_t_s);
		recv(sock_t, buff_i, sizeof buff_i, 0);
		char type = buff_i[5];

		putchar(type);
		putchar('\n');

		if (type == 'V') {
			FILE *hand = popen("./nihonrender/render | ffmpeg -loglevel 0 -i - -f gif -sws_dither bayer -", "r");
			memset(buff_o + head_l, 0, sizeof buff_o - head_l);
			fread(buff_o + head_l, sizeof buff_o - head_l, 1, hand);
			pclose(hand);

			char *term = buff_o + sizeof buff_o - 1;
			while (*term-- != ';') {}

			send(sock_t, buff_o, (size_t)(term - buff_o + 2), 0);
		}
		else {
			if (type == 'R') {
				ry += 0.78539816339f;
			}
			if (type == 'L') {
				ry -= 0.78539816339f;
			}
			if (type == 'U') {
				cx -= 2*sinf(ry);
				cz += 2*cosf(ry);
			}
			if (type == 'D') {
				cx += sinf(ry);
				cz -= cosf(ry);
			}

			send(sock_t, "HTTP/1.1 302 Found\r\nConnection: close\r\nLocation: https://github.com/blocksrey\r\n\r\nWHAT THE FUCK", 94, 0);
		}

		close(sock_t);
	}
}
