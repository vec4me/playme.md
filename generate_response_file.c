#define GENERATE_RESPONSE_FILE
#include "generate_response.h"

// make.sh makes the make.c which makes the server of your choice ok cool
int main(int argc, char **argv) {
	(void)argc; // wow that's interesting

	Response response = generate_response(argv[1]);
	free(response.content);

	return 0;
}
