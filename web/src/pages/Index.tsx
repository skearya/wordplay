import { generalEmitter, lobbyEmitter, useEventEmitter } from "../lib/events";
import type { ClientLobby } from "@bindings/ClientLobby";
import type { ClientMessage } from "@bindings/ClientMessage";
import type { LobbyState } from "@bindings/LobbyState";
import type { ServerClient } from "@bindings/ServerClient";
import type { ServerMessage } from "@bindings/ServerMessage";
import type { ServerState } from "@bindings/ServerState";
import type { SocketParams } from "@bindings/SocketParams";
import { useEffect, useState } from "react";

export function Index() {
	const [state, setState] = useState<
		| { kind: "loading" }
		| {
				kind: "connected";
				state: ServerState;
				clients: Clients;
		  }
	>({ kind: "loading" });

	const emitter = useEventEmitter<ServerMessage>();

	useEffect(() => {
		const room = "one";

		const params: SocketParams = {
			username: "Client",
			rejoinToken: null,
		};

		const urlParams = new URLSearchParams(
			Object.entries(params).filter(
				(param): param is [string, string] => param[1] !== null,
			),
		);

		const socket = new WebSocket(
			`ws://localhost:3000/${room}?${urlParams}`,
		);

		socket.addEventListener("open", () => {
			console.log("Connected");
		});

		socket.addEventListener("message", (e) => {
			const message: ServerMessage = JSON.parse(e.data);
			console.log("Message", message);

			switch (message.kind) {
				case "general":
					generalEmitter.emit(message.data)
					break;
				case "lobby":
					lobbyEmitter.emit(message.data)
					break;
				case "inGame":
					inGameEmitter.emit(message.data)
					break;
				default:
					message satisfies never;
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

	return state.kind === "loading" ? (
		<h1>Loading</h1>
	) : state.kind === "connected" ? (
		<Connected {...state} />
	) : (
		(state satisfies never)
	);
}

type Clients = {
	[uuid in string]?: ServerClient;
};

type ConnectedProps = {
	// state: ServerState;
	// clients: Clients;
	// sender: (message: ClientMessage) => void;
};

function Connected({ state, clients, sender }: ConnectedProps) {
	return state.kind === "lobby" ? (
		<Lobby
			state={state}
			clients={clients}
			sender={(data) => sender({ kind: "lobby", data })}
		/>
	) : state.kind === "game" ? (
		<></>
	) : (
		(state satisfies never)
	);
}

type LobbyProps = {};

function Lobby({ state: { ready, timerStart }, clients, sender }: LobbyProps) {
	return (
		<div>
			<h1>lobby</h1>
			<div>{ready.map((client) => clients[client]!.username)}</div>
			<button onClick={() => sender({ kind: "ready" })}>ready</button>
			{timerStart ? <h1>(countdown started)</h1> : null}
		</div>
	);
}
