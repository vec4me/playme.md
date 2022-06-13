#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
	(void)argc;

	FILE *input = fopen(argv[1], "r");
	fseek(input, 0, SEEK_END);
	size_t input_length = (size_t)ftell(input);

	char *buffer = malloc(256);
	sprintf(buffer, "HTTP/1.1 200 OK\r\nCache-Control: max-age=0\r\nConnection: close\r\nContent-Length: %li\r\nContent-Type: image/jpeg\r\n\r\n", input_length);
	size_t header_length = strlen(buffer);
	size_t buffer_length = header_length + input_length;
	buffer = realloc(buffer, buffer_length);

	rewind(input);
	fread(buffer + header_length, sizeof(char), input_length, input);

	FILE *output = fopen("content.h", "w");
	fprintf(output, "static unsigned char content[]={");
	for (size_t i = 0; i < buffer_length; ++i)
		fprintf(output, "0x%x,", (unsigned char)buffer[i]);
	fprintf(output, "};\nstatic size_t content_length=%zu;\n", buffer_length);
	fclose(output);

	return 0;
}
