#ifndef PPM_H
#define PPM_H

#include <stdio.h>

static void writePPM(float buffer[][3], int width, int height, char *path) {
	FILE *handle = fopen(path, "w");

	fprintf(handle, "P6\n%i %i\n255\n", width, height);

	for (int i = 0; i < width*height; ++i) fprintf(
		handle,
		"%c%c%c",
		(char)(255*buffer[i][0]),
		(char)(255*buffer[i][1]),
		(char)(255*buffer[i][2])
	);

	fclose(handle);
}

#endif
