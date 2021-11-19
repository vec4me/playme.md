forward
w, x, y, z = q:dump();
w *= H;
x *= H;
y *= H;
z *= H;

return {
	i*(1 - y*y - z*z) + j*(x*y - w*z) + k*(x*z + w*y),
	i*(x*y + w*z) + j*(1 - x*x - z*z) + k*(y*z - w*x),
	i*(x*z - w*y) + j*(y*z + w*x) + k*(1 - x*x - y*y)
};



inverse
w, x, y, z = q:dump();
w *= H;
x *= H;
y *= H;
z *= H;

return {
	i*(1 - y*y - z*z) + j*(x*y + w*z) + k*(x*z - w*y),
	i*(x*y - w*z) + j*(1 - x*x - z*z) + k*(y*z + w*x),
	i*(x*z + w*y) + j*(y*z - w*x) + k*(1 - x*x - y*y)
};