#ifndef PPM_H
#define PPM_H

#include <stdio.h>

static void writePPM(int *buffer, int width, int height, char *path) {
	FILE *handle = fopen(path, "w");

	fprintf(handle, "P6\n%i %i\n255\n", width, height);

	for (int i = 0; i < width*height; ++i) fprintf(
		handle,
		"%c%c%c",
		buffer[i]&255,
		buffer[i]>>8&255,
		buffer[i]>>16&255
	);

	fclose(handle);
}

#endif
