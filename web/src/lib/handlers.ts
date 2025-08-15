import type { Info } from '@bindings/Info';
import type { ServerMessage } from '@bindings/ServerMessage';

function handleServerMessage(info: Info, message: ServerMessage) {
	switch (message.kind) {
		case 'general':
			break;
		case 'lobby':
			break;
		case 'game':
			break;
	}
}
