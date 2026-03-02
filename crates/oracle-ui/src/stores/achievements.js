import { writable } from 'svelte/store';

export const achievements = writable([
    { id: 1, name: 'Shardbearer Godrick', desc: 'Defeated Shardbearer Godrick', icon: '👑', unlocked: true },
    { id: 2, name: 'First Steps', desc: 'Complete the tutorial', icon: '🎯', unlocked: true },
    { id: 3, name: 'Elden Ring', desc: 'Obtained all achievements', icon: '⭐', unlocked: false },
]);

export function unlockAchievement(achievementId) {
    achievements.update(achs =>
        achs.map(ach =>
            ach.id === achievementId
                ? { ...ach, unlocked: true }
                : ach
        )
    );
}