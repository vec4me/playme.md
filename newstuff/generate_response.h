#include <stdio.h>
#include <stdlib.h>

typedef struct {
	size_t content_length;
	char *content;
} Response;

static Response generate_response(char *path) {
	char header[] = "HTTP/1.1 200 OK\r\nCache-Control: max-age=0\r\nConnection: close\r\nContent-Type: image/gif\r\n\r\n";
	size_t header_length = sizeof header - 1; // we want the length of the header, not the string

	FILE *body = fopen(path, "r");
	fseek(body, 0, SEEK_END);
	size_t body_length = (size_t)ftell(body);

	Response response;
	response.content_length = header_length + body_length;
	response.content = malloc(response.content_length);
	sprintf(response.content, "%s", header); // write header
	rewind(body); // rewind the shit...
	fread(response.content + header_length, sizeof(char), body_length, body); // write body

	#ifdef GENERATE_RESPONSE_FILE
		FILE *file = fopen("response.h", "w");
		fprintf(file, "static unsigned char response[]={");
		for (size_t index = 0; index < response.content_length; ++index)
			fprintf(file, "0x%x,", (unsigned char)response.content[index]);
		fprintf(file, "};\nstatic size_t response_length=%zu;\n", response.content_length);
		fclose(file);
	#endif

	return response; // make sure you free this shit
}
