import Phaser from 'phaser';
import { BattleScene } from './scenes/BattleScene.js';
import { MatterScene } from './scenes/MatterScene.js';
import { assertVersionSync, initGameLogic } from './game-logic.js';
import { APP_VERSION } from './version.js';
import { mountDebugOverlay, toggleDebugUi } from './debug-overlay.js';
import type { UserSettingsDto } from '$lib/auth/auth.js';

export async function createBattleGame(
	gameEl: HTMLDivElement,
	hudEl: HTMLDivElement,
	userSettings?: UserSettingsDto,
): Promise<Phaser.Game> {
	await initGameLogic();
	assertVersionSync();

	const badge = document.createElement('div');
	badge.id = 'version-badge';
	badge.textContent = `v${APP_VERSION}`;
	document.body.appendChild(badge);

	document.title = import.meta.env.DEV ? 'Battle Control DEV' : 'Battle Control';

	mountDebugOverlay();
	window.addEventListener('keydown', (event) => {
		if (event.code !== 'Backquote') return;
		event.preventDefault();
		toggleDebugUi();
	});

	const initialScene = window.location.hash === '#matter' ? MatterScene : BattleScene;
	const config: Phaser.Types.Core.GameConfig = {
		type: Phaser.AUTO,
		width: gameEl.clientWidth,
		height: gameEl.clientHeight,
		parent: gameEl,
		backgroundColor: '#000000',
		scene: [initialScene],
		scale: {
			mode: Phaser.Scale.RESIZE,
			autoCenter: Phaser.Scale.CENTER_BOTH,
		},
		physics: {
			default: 'matter',
			matter: {
				gravity: { x: 0, y: 0 },
				debug: false,
			},
		},
	};

	const game = new Phaser.Game(config);
	game.registry.set('hudElement', hudEl);
	if (userSettings) {
		game.registry.set('userSettings', userSettings);
	}
	return game;
}
