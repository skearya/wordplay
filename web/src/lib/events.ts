import type { ServerAnagrams } from '@bindings/ServerAnagrams';
import type { ServerCore } from '@bindings/ServerCore';
import type { ServerGame } from '@bindings/ServerGame';
import type { ServerGeneral } from '@bindings/ServerGeneral';
import type { ServerLobby } from '@bindings/ServerLobby';
import type { ServerWordBomb } from '@bindings/ServerWordBomb';
import { EventEmitter } from './event-emitter';

export const coreEmitter = new EventEmitter<ServerCore>();
export const generalEmitter = new EventEmitter<ServerGeneral>();
export const lobbyEmitter = new EventEmitter<ServerLobby>();
export const gameEmitter = new EventEmitter<ServerGame>();
export const wordBombEmitter = new EventEmitter<ServerWordBomb>();
export const anagramsEmitter = new EventEmitter<ServerAnagrams>();
