#ifndef PPM_H
#define PPM_H

#include <stdio.h>

static void writePPM(int *fb, int w, int h, char *path) {
	FILE *handle = fopen(path, "w");

	fprintf(handle, "P6\n%i %i\n255\n", w, h);

	for (int i = 0; i < w*h; ++i) fprintf(
		handle,
		"%c%c%c",
		fb[i]&255,
		fb[i]>>8&255,
		fb[i]>>16&255
	);
}

#endif
