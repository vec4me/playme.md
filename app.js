const http = require("http");
const fs = require("fs");
const {exec} = require("child_process");

var count = Number(fs.readFileSync("count"));

var actions = {};

actions.right = function(response) {
	exec("curl -X PURGE https://camo.githubusercontent.com/b7e1cfd4c01b688c299f32a84dfe2995590f4e9ecec5bdc67ef6644d2b13ef11/687474703a2f2f7775682e626c6f636b737265792e636f6d3a353637392f72656e646572");

	response.writeHead(302, {"Location": "https://github.com/Blocksrey"});

	response.end();
}

actions.render = function(response) {
	response.setHeader("Cache-Control", "no-cache");
	response.setHeader("Content-Type", "image/jpg");

	response.end(fs.readFileSync("draw/baked/" + Math.floor(32*Math.random()) + ".jpg"));
}

http.createServer((request, response) => {
	console.log(request.url);

	fs.writeFileSync("count", (++count).toString());

	response.removeHeader("Date");
	response.removeHeader("Connection");
	response.removeHeader("Keep-Alive");

	var call = actions[request.url.substr(1)]
	if (call)
		call(response);

}).listen(5679);