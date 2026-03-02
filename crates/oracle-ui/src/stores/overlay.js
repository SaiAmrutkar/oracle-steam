import { writable } from 'svelte/store';

export const overlayVisible = writable(false);
export const gamePaused = writable(false);
export const activePanel = writable(null);

export function toggleOverlay() {
    overlayVisible.update(v => !v);
    gamePaused.update(p => !p);
}

export function openPanel(panelName) {
    activePanel.set(panelName);
}

export function closePanel() {
    activePanel.set(null);
}