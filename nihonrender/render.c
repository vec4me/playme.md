#include <stdlib.h>
#include <stdio.h>
#include <math.h>

#define w 320
#define h 200

static int pixels[w*h];

static int CUR_COLOR = 0;

#define SET_COLOR(R, G, B) CUR_COLOR = R | G<<8 | B<<16
#define COLOR_PIXEL(X, Y, w) pixels[X + Y*w] = CUR_COLOR

#define MIX(A, B, X) A + X*(B - A)
#define DOT(AX, AY, AZ, BX, BY, BZ) AX*BX + AY*BY + AZ*BZ

static int PL_sin[256] = {
	0x0000, 0x0324, 0x0647, 0x096a, 0x0c8b, 0x0fab, 0x12c8, 0x15e2,
	0x18f8, 0x1c0b, 0x1f19, 0x2223, 0x2528, 0x2826, 0x2b1f, 0x2e11,
	0x30fb, 0x33de, 0x36ba, 0x398c, 0x3c56, 0x3f17, 0x41ce, 0x447a,
	0x471c, 0x49b4, 0x4c3f, 0x4ebf, 0x5133, 0x539b, 0x55f5, 0x5842,
	0x5a82, 0x5cb4, 0x5ed7, 0x60ec, 0x62f2, 0x64e8, 0x66cf, 0x68a6,
	0x6a6d, 0x6c24, 0x6dca, 0x6f5f, 0x70e2, 0x7255, 0x73b5, 0x7504,
	0x7641, 0x776c, 0x7884, 0x798a, 0x7a7d, 0x7b5d, 0x7c29, 0x7ce3,
	0x7d8a, 0x7e1d, 0x7e9d, 0x7f09, 0x7f62, 0x7fa7, 0x7fd8, 0x7ff6,
	0x8000, 0x7ff6, 0x7fd8, 0x7fa7, 0x7f62, 0x7f09, 0x7e9d, 0x7e1d,
	0x7d8a, 0x7ce3, 0x7c29, 0x7b5d, 0x7a7d, 0x798a, 0x7884, 0x776c,
	0x7641, 0x7504, 0x73b5, 0x7255, 0x70e2, 0x6f5f, 0x6dca, 0x6c24,
	0x6a6d, 0x68a6, 0x66cf, 0x64e8, 0x62f2, 0x60ec, 0x5ed7, 0x5cb4,
	0x5a82, 0x5842, 0x55f5, 0x539b, 0x5133, 0x4ebf, 0x4c3f, 0x49b4,
	0x471c, 0x447a, 0x41ce, 0x3f17, 0x3c56, 0x398c, 0x36ba, 0x33de,
	0x30fb, 0x2e11, 0x2b1f, 0x2826, 0x2528, 0x2223, 0x1f19, 0x1c0b,
	0x18f8, 0x15e2, 0x12c8, 0x0fab, 0x0c8b, 0x096a, 0x0647, 0x0324
};

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

	float ry = 12314;

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

	float fk = 0.000000001f;

	float fl = (1 - powf(fk, cy))/logf(fk);
	float fh = (powf(fk, 10000) - 1)/logf(fk);

	float AR = 0.941176f;
	float AG = 0.929412f;
	float AB = 0.854902f;

	printf("P6\n%i %i\n255\n", w, h);

	for (int i = w*h; i--;) {
		int ix = i%w;
		int iy = i/w;

		float tx = (w - 2*(float)ix)/h;
		float ty = 2*(float)iy/h - 1;

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

			float clouds = sinf(0.005f*hx + 10*sinf(0.0005f*hz));
			if (clouds > 0) {
				R = 1 - 0.2f*clouds;
				G = 1 - 0.2f*clouds;
				B = 1 - 0.2f*clouds;
			}
		}

		SET_COLOR(
			(int)(255*(MIX(R, AR, f))),
			(int)(255*(MIX(G, AG, f))),
			(int)(255*(MIX(B, AB, f)))
		);

		COLOR_PIXEL(ix, iy, w);

		putchar(pixels[i]&255);
		putchar(pixels[i]>>8&255);
		putchar(pixels[i]>>16&255);
	}

	return 0;
}
