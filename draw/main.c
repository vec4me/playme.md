#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include "int.h"
#include "ppm.h"

#define W 512
#define H 256

#define PI 3.1415927f
#define TAU 6.2831853f

float pixels[W*H][3];

float CR, CG, CB;

#define COLOR_PIXEL(X, Y)\
	pixels[X + W*(Y)][0] = CR;\
	pixels[X + W*(Y)][1] = CG;\
	pixels[X + W*(Y)][2] = CB

#define SET_COLOR(R, G, B)\
	CR = R;\
	CG = G;\
	CB = B

#define LENGTH(a) (sizeof(a)/sizeof(*a))
#define DOT(x0, y0, x1, y1) ((x0)*(x1) + (y0)*(y1))
#define DD(x0, y0) ((x0)*(x0) + (y0)*(y0))
#define CLAMP(n, a, b) fmaxf(a, fminf(n, b))

float isqrt(float n) {
	const float x = 0.5f*n;
	const float t = 1.5f;
	union {
		float f;
		u32 i;
	} c = {.f = n};
	c.i = 0x5f3759df - (c.i >> 1);
	c.f *= t - x*c.f*c.f;
	return c.f;
}

void line(
	u16 *o,
	s16 *d
) {
	u16 ox = o[0];
	u16 oy = o[1];

	s16 dx = d[0];
	s16 dy = d[1];

	float i = isqrt(dx*dx + dy*dy);

	for (float u = 0; u < 1; u += i) {
		COLOR_PIXEL(
			ox + (s16)(dx*u),
			oy + (s16)(dy*u)
		);
	}
}

int main() {
	u16 nf = 32;

	char fp[24];

	u16 px, py;

	u16 o[2];
	s16 d[2];

	for (u16 i = 0; i < nf; ++i) {
		sprintf(fp, "raw/%u.ppm", i);

		SET_COLOR(0, 0, 0);

		for (u32 p = 0; p < W*H; ++p) {
			px = p%W;
			py = p/W;

			SET_COLOR((1 + sinf(0.1*px))/2, (1 + sinf(0.1*py))/2, 0);

			COLOR_PIXEL(px, py);
		}

		SET_COLOR(0.3, 0.3, 0);

		o[0] = rand()%W/2;
		o[1] = rand()%H/2;

		d[0] = rand()%W/2;
		d[1] = rand()%H/2;

		line(o, d);

		writePPM(pixels, W, H, fp);
	}

	return 0;
}