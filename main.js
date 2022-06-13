import {createServer} from 'http'
import {readFileSync, writeFileSync} from 'fs'
import {exec} from 'child_process'














var PI   = Math.PI,
	sin  = Math.sin,
	cos  = Math.cos,
	tan  = Math.tan,
	asin = Math.asin,
	atan = Math.atan2,
	acos = Math.acos,
	rad  = PI / 180;

// sun calculations are based on http://aa.quae.nl/en/reken/zonpositie.html formulas


// date/time constants and conversions

var dayMs = 1000 * 60 * 60 * 24,
	J1970 = 2440588,
	J2000 = 2451545;

function toJulian(date) { return date.valueOf() / dayMs - 0.5 + J1970; }
function fromJulian(j)  { return new Date((j + 0.5 - J1970) * dayMs); }
function toDays(date)   { return toJulian(date) - J2000; }


// general calculations for position

var e = rad * 23.4397; // obliquity of the Earth

function rightAscension(l, b) { return atan(sin(l) * cos(e) - tan(b) * sin(e), cos(l)); }
function declination(l, b)    { return asin(sin(b) * cos(e) + cos(b) * sin(e) * sin(l)); }

function azimuth(H, phi, dec)  { return atan(sin(H), cos(H) * sin(phi) - tan(dec) * cos(phi)); }
function altitude(H, phi, dec) { return asin(sin(phi) * sin(dec) + cos(phi) * cos(dec) * cos(H)); }

function siderealTime(d, lw) { return rad * (280.16 + 360.9856235 * d) - lw; }

function astroRefraction(h) {
	if (h < 0) // the following formula works for positive altitudes only.
		h = 0; // if h = -0.08901179 a div/0 would occur.

	// formula 16.4 of "Astronomical Algorithms" 2nd edition by Jean Meeus (Willmann-Bell, Richmond) 1998.
	// 1.02 / tan(h + 10.26 / (h + 5.10)) h in degrees, result in arc minutes -> converted to rad:
	return 0.0002967 / Math.tan(h + 0.00312536 / (h + 0.08901179));
}

// general sun calculations

function solarMeanAnomaly(d) { return rad * (357.5291 + 0.98560028 * d); }

function eclipticLongitude(M) {

	var C = rad * (1.9148 * sin(M) + 0.02 * sin(2 * M) + 0.0003 * sin(3 * M)), // equation of center
		P = rad * 102.9372; // perihelion of the Earth

	return M + C + P + PI;
}

function sunCoords(d) {

	var M = solarMeanAnomaly(d),
		L = eclipticLongitude(M);

	return {
		dec: declination(L, 0),
		ra: rightAscension(L, 0)
	};
}

function getPosition(date, lat, lng) {

	var lw  = rad * -lng,
		phi = rad * lat,
		d   = toDays(date),

		c  = sunCoords(d),
		H  = siderealTime(d, lw) - c.ra;

		console.log(c.dec*180/Math.PI, c.ra*180/Math.PI)

	return {
		azimuth: azimuth(H, phi, c.dec),
		altitude: altitude(H, phi, c.dec)
	};
};





let asd = getPosition(new Date(), 0, 0)
console.log(asd.azimuth*180/Math.PI, asd.altitude*180/Math.PI)












let state = JSON.parse(readFileSync('state'))

let close = () => {
	console.log('ahhh fuck we saving this bruh')

	writeFileSync('state', JSON.stringify(state))

	process.exit()
}

process.on('SIGINT', close)
process.on('SIGTERM', close)

let calcTime = offset => {
	let here = new Date()

	let there = new Date(here.getTime() + 60000*(here.getTimezoneOffset() + 60*offset))

	return (there.getHours() + there.getMinutes()/60)/24
}

let getSunDirection = () => {
	let s0 = 4/24
	let s1 = 16/24

	let t = Math.PI*(calcTime(9) - s0)/(s1 - s0)

	return [
		-Math.cos(t),
		Math.sin(t),
		0
	]
}

let decache = response => {
	response.writeHead(302, {'Location': 'https://github.com/Blocksrey'})

	response.end()
}

let actions = {}

actions.R = response => {
	decache(response)

	state.ry += Math.PI/4
}

actions.L = response => {
	decache(response)

	state.ry -= Math.PI/4
}

actions.U = response => {
	decache(response)

	state.px -= 2*Math.sin(state.ry)
	state.pz += 2*Math.cos(state.ry)
}

actions.D = response => {
	decache(response)

	state.px += Math.sin(state.ry)
	state.pz -= Math.cos(state.ry)
}

actions.V = response => {
	response.setHeader('Connection', 'close')
	response.setHeader('Content-Type', 'image/gif')

	let sequence = [
		'cd draw; ./render',
		state.px, state.pz,
		state.ry,
		...getSunDirection(),
		'; ./cook'
	]

	exec(
		sequence.join(' '),
		(_, ...args) => {
			console.log(...args)
			response.end(readFileSync('draw/baked/cool.gif'))
		}
	)
}

createServer((request, response) => {
	response.removeHeader('Date')
	response.removeHeader('Connection')
	response.removeHeader('Keep-Alive')

	console.log(request.url)

	let action = actions[request.url.substr(1)]
	if (action) {
		++state.count

		action(response)
	}

}).listen(7890)
