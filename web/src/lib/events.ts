import type { ServerAnagrams } from '@bindings/ServerAnagrams';
import type { ServerGame } from '@bindings/ServerGame';
import type { ServerLobby } from '@bindings/ServerLobby';
import type { ServerMessage } from '@bindings/ServerMessage';
import type { ServerWordBomb } from '@bindings/ServerWordBomb';
import { EventEmitter } from './eventemitter';

export const rootEmitter = new EventEmitter<ServerMessage>();
export const lobbyEmitter = new EventEmitter<ServerLobby>();
export const gameEmitter = new EventEmitter<ServerGame>();
export const wordBombEmitter = new EventEmitter<ServerWordBomb>();
export const anagramsEmitter = new EventEmitter<ServerAnagrams>();
