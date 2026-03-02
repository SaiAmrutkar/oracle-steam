import { writable } from 'svelte/store';

export const messages = writable([
    { id: 1, sender: 'Robert Fakename', text: 'Hey', self: false },
    { id: 2, sender: 'You', text: 'Up for a run?', self: true },
]);

export function sendMessage(text) {
    messages.update(msgs => [
        ...msgs,
        {
            id: Date.now(),
            sender: 'You',
            text,
            self: true,
        }
    ]);
}

export function receiveMessage(sender, text) {
    messages.update(msgs => [
        ...msgs,
        {
            id: Date.now(),
            sender,
            text,
            self: false,
        }
    ]);
}