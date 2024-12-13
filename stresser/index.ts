import type {
  ClientMessage,
  ServerMessage,
} from "../client/src/lib/types/messages.ts";

const words = (
  await Bun.file("../server/src/static/words_alpha.txt").text()
).split("\n");

let i = 0;

while (true) {
  console.log(i);
  room(i.toString());
  i += 1;

  await Bun.sleep(50);
}

async function room(name: string) {
  const owner = player(name, "Owner");
  const clients = Array.from({ length: 4 }, (_, i) =>
    player(name, i.toString())
  );

  await owner.connect();
  await Promise.all(clients.map((client) => client.connect()));
  await Bun.sleep(1000);

  while (true) {
    owner.ready();
    clients.forEach((client) => client.ready());
    await Bun.sleep(1000);

    owner.startEarly();
    await Bun.sleep(1000);

    await owner.waitUntilFinished();
    await Bun.sleep(1000);
  }
}

function player(room: string, username: string) {
  let socket: WebSocket | undefined = undefined;
  let uuid: string | undefined = undefined;

  const send = (message: ClientMessage) =>
    socket!.send(JSON.stringify(message));

  const onTurn = (prompt: string) => {
    if (Math.random() > 0.5) {
      Bun.sleep(1000).then(() =>
        send({
          type: "WordBombGuess",
          word: answerPrompt(prompt)!,
        })
      );
    }
  };

  return {
    connect: () => {
      return new Promise((resolve, reject) => {
        socket = new WebSocket(
          `ws://localhost:3021/api/room/${room}?username=${username}`
        );

        socket.addEventListener("message", (event) => {
          const data: ServerMessage = JSON.parse(event.data);

          switch (data.type) {
            case "Pong":
              console.log(`Ping: ${Date.now() - data.timestamp}ms`);
              break;
            case "Info":
              uuid = data.uuid;
              resolve(undefined);
              break;
            case "Error":
              console.log("Error: " + data.content);
              break;
            case "GameStarted":
              const game = data.game as Extract<
                Extract<ServerMessage, { type: "GameStarted" }>["game"],
                { type: "WordBomb" }
              >;

              if (game.turn === uuid) {
                onTurn(game.prompt);
              }

              break;
            case "GameEnded":
              break;
            case "WordBombPrompt":
              if (data.turn === uuid) {
                onTurn(data.prompt);
              }

              break;
          }
        });

        socket.addEventListener("close", () => {
          reject(Error("closed"));
        });

        socket.addEventListener("error", () => {
          reject(Error("errored"));
        });

        setTimeout(() => send({ type: "Ping", timestamp: Date.now() }), 10000);
      });
    },
    ready: () => {
      send({ type: "Ready" });
    },
    startEarly: () => {
      send({ type: "StartEarly" });
    },
    waitUntilFinished: () => {
      return new Promise((resolve) => {
        const controller = new AbortController();
        const signal = controller.signal;

        socket?.addEventListener(
          "message",
          (event) => {
            if (
              (JSON.parse(event.data) as ServerMessage).type === "GameEnded"
            ) {
              controller.abort();
              resolve(undefined);
            }
          },
          { signal }
        );
      });
    },
  };
}

function answerPrompt(prompt: string) {
  for (let i = 0; i < words.length; i++) {
    if (words[i].includes(prompt)) {
      return prompt;
    }
  }
}
