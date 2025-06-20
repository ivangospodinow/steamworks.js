import * as steamworks from "steamworks2.js";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
}
