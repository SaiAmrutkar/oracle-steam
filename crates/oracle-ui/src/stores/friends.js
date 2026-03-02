import { writable } from 'svelte/store';

export const friends = writable([
    { id: 1, name: 'Robert Fakename', status: 'ingame', game: 'Team Fortress 2' },
    { id: 2, name: 'Marcia Fakename', status: 'ingame', game: 'CS:GO' },
    { id: 3, name: 'xXxSlayerGod420xXx', status: 'ingame', game: 'Cyberpunk 2077' },
    { id: 4, name: 'LessShopping', status: 'online', game: null },
    { id: 5, name: 'BaccaratKing', status: 'online', game: null },
]);

export function addFriend(friend) {
    friends.update(f => [...f, friend]);
}

export function removeFriend(friendId) {
    friends.update(f => f.filter(friend => friend.id !== friendId));
}