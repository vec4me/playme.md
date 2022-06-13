#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <netinet/in.h>
#include "content.h"
#include <string.h>

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
	char read_buffer[1024] = {0};
	char *find;
	char action;

	for (;;) {
		them_sock = accept(us_sock, (struct sockaddr *)&them_info, &them_info_size);
		recv(them_sock, read_buffer, sizeof read_buffer, 0);
		find = strchr(read_buffer, '/');
		action = find ? find[1] : 'V';
		if (action == 'V')
			send(them_sock, content, content_length, 0);
		else {
			if (action == 'R')
				printf("R\n");
			if (action == 'L')
				printf("L\n");
			if (action == 'U')
				printf("U\n");
			if (action == 'D')
				printf("D\n");
			send(them_sock, "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nTHANKS BITCH", 49, 0);
		}
		close(them_sock);
	}
}
