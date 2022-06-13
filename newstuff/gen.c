#include <stdio.h>
#include <stdlib.h>

#include <string.h>

static size_t getHandleLength(FILE *handle) {
    fseek(handle, 0, SEEK_END);
    return (size_t)ftell(handle);
}

static void readHandle(FILE *handle, char *buffer, size_t handleLength) {
	rewind(handle);
    fread(buffer, sizeof(char), handleLength, handle);
}

int main(int argc, char **argv) {
	FILE *handle = fopen(argv[1], "r");
    // get length of file contents
    size_t handleLength = getHandleLength(handle);

    // create a buffer to be sent
    char *content = malloc(256);
    // append the header info
    sprintf(content, "HTTP/1.1 200 OK\r\nCache-Control: max-age=0\r\nConnection: close\r\nContent-Length: %li\r\nContent-Type: image/png\r\n\r\n", handleLength);
    // get the current length of the string
    size_t templateLength = strlen(content);
    // reallocate enough memory for file contents
    content = realloc(content, templateLength + handleLength);
    // append file contents to content buffer (header + content)
    readHandle(handle, content + templateLength, handleLength);

	size_t contentLength = templateLength + handleLength;

	FILE *out = fopen("content.h", "w");
	fprintf(out, "static unsigned char content[]={");
	for (size_t i = 0; i < contentLength; ++i)
		fprintf(out, "0x%x,", (unsigned char)content[i]);
	fprintf(out,"};\nstatic size_t contentLength=%zu;\n", contentLength);
	fclose(out);

	return 0;
}
