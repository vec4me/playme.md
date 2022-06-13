#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <netinet/in.h>
#include "content.h"

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

	for (;;) {
		them_sock = accept(us_sock, (struct sockaddr *)&them_info, &them_info_size);
		send(them_sock, content, content_length, 0);
		recv(them_sock, read_buffer, sizeof read_buffer, 0);
		printf("%s\n", read_buffer);
		close(them_sock);
	}
}
