#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <netinet/in.h>
#include <string.h>
#include <math.h>
#include "generate_response.h"

int main() {
	struct sockaddr_in us_info;
	us_info.sin_family = AF_INET;
	us_info.sin_port = htons(7890);
	us_info.sin_addr.s_addr = INADDR_ANY;

	int us_sock = socket(AF_INET, SOCK_STREAM, 0);
	bind(us_sock, (struct sockaddr *)&us_info, sizeof us_info);
	listen(us_sock, 0);

	struct sockaddr_in them_info;
	unsigned them_info_size = sizeof them_info;

	int them_sock;
	char read_buffer[1024];
	Response response;

	char run_string[128];

	float cx = 2000.0f;
	float cz = 2000.0f;

	float ry = 0.0f;

	float sdx = 1.0f;
	float sdy = 0.0f;
	float sdz = 0.0f;

	for (;;) {
		them_sock = accept(us_sock, (struct sockaddr *)&them_info, &them_info_size);
		recv(them_sock, read_buffer, sizeof read_buffer, 0);

		char *action = read_buffer;
		for (; *action++ != '/';) {} // search for the "/" and get the char after
		printf("%c\n", *action);

		if (*action == 'V') {
			sprintf(run_string, "cd draw; ./render %f %f %f %f %f %f; ./cook.sh", (double)cx, (double)cz, (double)ry, (double)sdx, (double)sdy, (double)sdz);
			printf("%s\n", run_string);
			system(run_string);

			response = generate_response("draw/baked/cool.gif");

			send(them_sock, response.content, response.content_length, 0);

			free(response.content);
		}
		else {
			if (*action == 'R') {
				ry += 0.78539816339f;
			}
			if (*action == 'L') {
				ry -= 0.78539816339f;
			}
			if (*action == 'U') {
				cx -= 2*sinf(ry);
				cz += 2*cosf(ry);
			}
			if (*action == 'D') {
				cx += sinf(ry);
				cz -= cosf(ry);
			}
			send(them_sock, "HTTP/1.1 302 Found\r\nConnection: close\r\nLocation: https://github.com/Blocksrey\r\n\r\nWHAT THE FUCK", 80, 0);
		}

		close(them_sock);
	}
}
