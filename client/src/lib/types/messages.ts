export type Uuid = string;

export type Games = "WordBomb" | "Anagrams";

export type ClientMessage =
  | { type: "Ping"; timestamp: number }
  | { type: "Ready" }
  | { type: "StartEarly" }
  | { type: "Unready" }
  | { type: "ChatMessage"; content: string }
  | { type: "PracticeRequest"; game: Games }
  | { type: "PracticeSubmission"; game: Games; prompt: string; input: string }
  | ({ type: "RoomSettings" } & RoomSettings)
  | { type: "WordBombInput"; input: string }
  | { type: "WordBombGuess"; word: string }
  | { type: "AnagramsGuess"; word: string };

export type ServerMessage =
  // lobby / generic
  | {
      type: "Pong";
      timestamp: number;
    }
  | {
      type: "Info";
      uuid: Uuid;
      room: RoomInfo;
    }
  | {
      type: "Error";
      content: string;
    }
  | {
      type: "ConnectionUpdate";
      uuid: Uuid;
      state: ConnectionUpdate;
    }
  | {
      type: "ChatMessage";
      author: Uuid;
      content: string;
    }
  | {
      type: "ReadyPlayers";
      ready: Array<Uuid>;
      countdown_update: CountdownState | null;
    }
  | {
      type: "StartingCountdown";
      time_left: number;
    }
  | {
      type: "PracticeSet";
      set: Array<string>;
    }
  | {
      type: "PracticeResult";
      correct: boolean;
    }
  | ({
      type: "RoomSettings";
    } & RoomSettings)
  | {
      type: "GameStarted";
      rejoin_token: string | null;
      game: Exclude<RoomStateInfo, { type: "Lobby" }>;
    }
  | {
      type: "GameEnded";
      new_room_owner: Uuid | null;
      info: PostGameInfo;
    }
  // word bomb
  | {
      type: "WordBombInput";
      uuid: Uuid;
      input: string;
    }
  | {
      type: "WordBombInvalidGuess";
      uuid: Uuid;
      reason: WordBombGuessInfo;
    }
  | {
      type: "WordBombPrompt";
      correct_guess: string | null;
      life_change: number;
      prompt: string;
      turn: Uuid;
    }
  // anagrams
  | {
      type: "AnagramsInvalidGuess";
      reason: AnagramsGuessInfo;
    }
  | {
      type: "AnagramsCorrectGuess";
      uuid: Uuid;
      guess: string;
      points: number;
    };

export type RoomInfo = {
  owner: Uuid;
  settings: RoomSettings;
  clients: Array<ClientInfo>;
  state: RoomStateInfo;
};

export type RoomSettings = {
  public: boolean;
  game: Games;
};

export type RoomStateInfo =
  | {
      type: "Lobby";
      ready: Array<Uuid>;
      starting_countdown: number | null;
    }
  | {
      type: "WordBomb";
      players: Array<WordBombPlayerData>;
      turn: Uuid;
      prompt: string;
      used_letters: Array<string> | null;
    }
  | {
      type: "Anagrams";
      players: Array<AnagramsPlayerData>;
      anagram: string;
    };

export type PostGameInfo =
  | {
      type: "WordBomb";
      winner: Uuid;
      mins_elapsed: number;
      words_used: number;
      fastest_guesses: Array<[Uuid, number]>;
      longest_words: Array<[Uuid, string]>;
      avg_wpms: Array<[Uuid, number]>;
      avg_word_lengths: Array<[Uuid, number]>;
    }
  | {
      type: "Anagrams";
      original_word: string;
      leaderboard: Array<[Uuid, number]>;
      used_words: Array<[Uuid, Array<string>]>;
    };

export type ClientInfo = {
  uuid: Uuid;
  username: string;
  disconnected: boolean;
};

type ConnectionUpdate =
  | {
      type: "Connected";
      username: string;
    }
  | {
      type: "Reconnected";
      username: string;
    }
  | {
      type: "Disconnected";
      new_room_owner: Uuid | null;
    };

type CountdownState =
  | {
      type: "InProgress";
      time_left: number;
    }
  | {
      type: "stopped";
    };

export type WordBombPlayerData = {
  uuid: Uuid;
  input: string;
  lives: number;
};

type WordBombGuessInfo =
  | {
      type: "PromptNotIn";
    }
  | {
      type: "NotEnglish";
    }
  | {
      type: "AlreadyUsed";
    };

export type AnagramsPlayerData = {
  uuid: Uuid;
  used_words: Array<string>;
};

export type AnagramsGuessInfo =
  | {
      type: "NotLongEnough";
    }
  | {
      type: "PromptMismatch";
    }
  | {
      type: "NotEnglish";
    }
  | {
      type: "AlreadyUsed";
    };
