#include <stdlib.h>
#include <math.h>
#include "ppm.h"

static int pixels[640*480];

static int CUR_COLOR = 0;

#define SET_COLOR(R, G, B) CUR_COLOR = R | G<<8 | B<<16
#define COLOR_PIXEL(X, Y, W) pixels[X + Y*W] = CUR_COLOR

#define MIX(A, B, X) A + X*(B - A)
#define DOT(AX, AY, AZ, BX, BY, BZ) AX*BX + AY*BY + AZ*BZ

static float isqrt(float n) {
	union {
		int i;
		float f;
	} u = {.f = n};

	u.i = 0x5f3759df - (u.i >> 1);
	u.f *= 0.5f*(3 - n*u.f*u.f);

	return u.f;
}

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

	float fk = 0.000000000000000001f;

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

		float f;

		float R;
		float G;
		float B;

		if (dy < 0) {
			float z = cy/-dy;

			float hx = cx + z*dx;
			float hz = cz + z*dz;

			f = fminf(fl/dy, 1);

			R = 0.3f;
			G = (1 + sinf(4*hx))/6 + 0.2f;
			B = 0;

			if ((int)hx%16 < 2 || (int)hz%24 < 1.5) {
				R = 0.2f;
				G = 0.2f;
				B = 0.2f;
			}
		}
		else {
			float z = 1000/dy;

			float hx = cx + z*dx;
			float hz = cz + z*dz;

			float l = DOT(dx, dy, dz, sdx, sdy, sdz);

			f = fminf(fh/dy, 1);

			R = 0.737f;
			G = 0;
			B = 0.176f;

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
		}

		SET_COLOR(
			(int)(255*(MIX(R, AR, f))),
			(int)(255*(MIX(G, AG, f))),
			(int)(255*(MIX(B, AB, f)))
		);

		COLOR_PIXEL(px, py, W);
	}

	writePPM(pixels, W, H, "raw/cool.ppm");

	return 0;
}
