import type { ServerMessage } from "@bindings/ServerMessage";
import type { SocketParams } from "@bindings/SocketParams";
import { useEffect } from "react";

export function Index() {
	useEffect(() => {
		const room = "one";

		const params: SocketParams = {
			username: "Client",
			rejoinToken: null,
		};

		const urlParams = new URLSearchParams(
			Object.entries(params).filter((param) => param[1] !== null) as [
				string,
				string,
			][],
		);

		const socket = new WebSocket(
			`ws://localhost:3000/${room}?${urlParams}`,
		);

		socket.addEventListener("open", () => {
			console.log("Connected");
		});

		socket.addEventListener("message", (e) => {
			const message: ServerMessage = JSON.parse(e.data);
			console.log(message);

			switch (message.kind) {
				case "general":
					switch (message.data.kind) {
						case "info":
							break;
						case "join":
							break;
						case "leave":
							break;
						case "pong":
							break;
						case "chat":
							break;
						case "settings":
							break;
						case "error":
							break;
						default:
							message.data satisfies never;
							break;
					}
					break;
				case "lobby":
					switch (message.data.kind) {
						case "ready":
							break;
						case "unready":
							break;
						case "practice":
							break;
						case "practiceResult":
							break;
						case "gameStarted":
							break;
						default:
							message.data satisfies never;
							break;
					}
					break;
				case "inGame":
					switch (message.data.kind) {
						case "endRequest":
							break;
						case "ended":
							break;
						default:
							message.data satisfies never;
							break;
					}
					break;
				default:
					message satisfies never;
					break;
			}
		});

		socket.addEventListener("error", (e) => {
			console.log("WebSocket error", e);
		});

		socket.addEventListener("close", (e) => {
			console.log("WebSocket closed", e);
		});

		return () => socket.close();
	}, []);

	return <h1>hi</h1>;
}
