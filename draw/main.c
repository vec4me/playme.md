#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <stdint.h>
#include "ppm.h"

static float pixels[800*600][3];

static float CR = 0;
static float CG = 0;
static float CB = 0;

#define COLOR_PIXEL(X, Y, W)\
	pixels[X + (W)*(Y)][0] = CR;\
	pixels[X + (W)*(Y)][1] = CG;\
	pixels[X + (W)*(Y)][2] = CB

#define SET_COLOR(R, G, B)\
	CR = R;\
	CG = G;\
	CB = B

#define MIX(a, b, x) ((a)*(1 - (x)) + (b)*(x))
#define DOT(x0, y0, z0, x1, y1, z1) ((x0)*(x1) + (y0)*(y1) + (z0)*(z1))

/*BLACK HOLE HAHA
static float isqrt(float n) {
	float f = n;
	int i = 0;

	i = 0x5f3759df - (i >> 1);
	f *= 0.5f*(3 - n*f*f);

	return f;
}
*/

static float isqrt(float n) {
	union {
		int i;
		float f;
	} u = {.f = n};

	u.i = 0x5f3759df - (u.i >> 1);
	u.f *= 0.5f*(3 - n*u.f*u.f);

	return u.f;
}

/*
static void line(
	int *o,
	int *d,
	int W
) {
	int ox = o[0];
	int oy = o[1];

	float dx = (float)d[0];
	float dy = (float)d[1];

	float i = isqrt(dx*dx + dy*dy);

	for (float u = 0; u < 1; u += i) {
		COLOR_PIXEL(
			ox + (int)(dx*u),
			oy + (int)(dy*u),
			W
		);
	}
}
*/

#include <time.h>

int main(int argc, char **argv) {
	float cx = 0;
	float cy = 1.72f;
	float cz = 0;

	float ry = 0;

	float sdx = 1;
	float sdy = 0;
	float sdz = 0;

	if (argc > 1) {
		cx = (float)atof(argv[1]);
		cz = (float)atof(argv[2]);

		ry = (float)atof(argv[3]);

		sdx = (float)atof(argv[4]);
		sdy = (float)atof(argv[5]);
		sdz = (float)atof(argv[6]);
	}

	float c_ry = cosf(ry);
	float s_ry = sinf(ry);

	float fk = 0.000000000000000000001f;

	float fl = (1 - powf(fk, cy))/logf(fk);
	float fh = (powf(fk, 10000) - 1)/logf(fk);

	float AR = 0.941176f;
	float AG = 0.929412f;
	float AB = 0.854902f;

	const int W = 640;
	const int H = 480;

	for (int p = W*H; p--;) {
		int px = p%W;
		int py = (int)(p/W);

		float tx = (W - 2*(float)px)/H;
		float ty = 1 - 2*(float)py/H;

		float dx = tx*c_ry - s_ry;
		float dy = ty;
		float dz = tx*s_ry + c_ry;

		float is = isqrt(dx*dx + dy*dy + dz*dz);

		dx *= is;
		dy *= is;
		dz *= is;

		if (dy < 0) {
			float z = cy/-dy;

			float hx = cx + z*dx;
			float hz = cz + z*dz;

			float f = fminf(fl/dy, 1);

			float R = 0.3f;
			float G = (1 + sinf(4*hx))/6 + 0.2f;
			float B = 0;

			if ((int)hx%16 < 2 || (int)hz%24 < 1.5) {
				R = 0.2f;
				G = 0.2f;
				B = 0.2f;
			}

			SET_COLOR(
				MIX(R, AR, f),
				MIX(G, AG, f),
				MIX(B, AB, f)
			);
		}
		else {
			float z = 1000/dy;

			float hx = cx + z*dx;
			float hz = cz + z*dz;

			float l = DOT(dx, dy, dz, sdx, sdy, sdz);

			float f = fminf(fh/dy, 1);

			float R = 0.737f;
			float G = 0;
			float B = 0.176f;

			if (l < 0.98f) {
				R = 0.5f - 0.5f*dy;
				G = 0.7f - 0.7f*dy;
				B = 1.0f - 0.3f*dy;
			}

			//*
			float clouds = sinf(0.005f*hx + 10*sinf(0.0005f*hz));
			if (clouds > 0) {
				R = 1 - 0.2f*clouds;
				G = 1 - 0.2f*clouds;
				B = 1 - 0.2f*clouds;
			}
			//*/

			SET_COLOR(
				MIX(R, AR, f),
				MIX(G, AG, f),
				MIX(B, AB, f)
			);
		}

		COLOR_PIXEL(px, py, W);
	}

	writePPM(pixels, W, H, "raw/cool.ppm");

	return 0;
}
