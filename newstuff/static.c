#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
	(void)argc; // damn that's cool

	char header[] = "HTTP/1.1 200 OK\r\nCache-Control: max-age=0\r\nConnection: close\r\nContent-Type: image/jpeg\r\n\r\n";
	size_t header_length = sizeof header - 1; // we want the length of the header, not the string

	FILE *body = fopen(argv[1], "r");
	fseek(body, 0, SEEK_END);
	size_t body_length = (size_t)ftell(body);

	char *content = malloc(header_length + body_length);
	sprintf(content, "%s", header); // write header
	rewind(body); // rewind the shit...
	fread(content + header_length, sizeof(char), body_length, body); // write body

	FILE *file = fopen("content.h", "w");
	fprintf(file, "static unsigned char content[]={");
	for (size_t index = 0; index < header_length + body_length; ++index)
		fprintf(file, "0x%x,", (unsigned char)content[index]);
	fprintf(file, "};\nstatic size_t content_length=%zu;\n", header_length + body_length);
	fclose(file);

	free(content); // free the memory

	return 0;
}
