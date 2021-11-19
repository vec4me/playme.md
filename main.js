import {createServer} from 'http'
import {readFileSync, writeFileSync} from 'fs'
import {exec} from 'child_process'

let state = JSON.parse(readFileSync('state'))

let close = () => {
	console.log('ahhh fuck we saving this nigga')
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

let purge = () =>
	exec('curl -X PURGE https://camo.githubusercontent.com/2c3a98caa31541e5309b804c4f496b847478315654aa4d7ed52cb55109620b0c/687474703a2f2f686f6d652e707975736f66742e636f6d3a353637392f72656e646572')

let decache = response => {
	purge()

	response.writeHead(302, {'Location': 'https://github.com/Blocksrey'})

	response.end()
}

setInterval(purge, 1000000)

let actions = {}

actions.right = response => {
	decache(response)

	state.ry += Math.PI/4
}

actions.left = response => {
	decache(response)

	state.ry -= Math.PI/4
}

actions.up = response => {
	decache(response)

	state.px -= 2*Math.sin(state.ry)
	state.pz += 2*Math.cos(state.ry)
}

actions.down = response => {
	decache(response)

	state.px += Math.sin(state.ry)
	state.pz -= Math.cos(state.ry)
}

actions.render = response => {
	response.setHeader('Cache-Control', 'no-cache')
	response.setHeader('Content-Type', 'image/jpg')

	let sequence = [
		'cd draw; ./render',

		state.px, state.pz,

		state.ry,

		...getSunDirection(),

		'; ./cook.sh'
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

}).listen(5679)