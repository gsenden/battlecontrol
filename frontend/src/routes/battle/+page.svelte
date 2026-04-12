<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { getCurrentUser, getUserSettings, type UserSettingsDto } from '$lib/auth/auth.js';
	import { completeGame, getGameInstance } from '$lib/games/games.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import type { BattleSetup } from '$lib/game/boot.js';

	let gameContainer: HTMLDivElement;
	let hudContainer: HTMLDivElement;
	let game: import('phaser').Game | null = null;
	let sceneReady = $state(false);
	let started = $state(false);
	let errorMessage = $state('');
	let activeGameId = $state<string | null>(null);
	let battleSetup = $state<BattleSetup | undefined>(undefined);
	let battleStarted = $state(false);
	let resultWinner = $state('');
	let battleCompleted = $state(false);

	onMount(() => {
		const onReady = () => {
			sceneReady = true;
			if (activeGameId) {
				triggerBattleStart();
			}
		};
		const onConnectionFailed = (event: Event) => {
			if (battleCompleted) {
				return;
			}
			if (activeGameId) {
				void goto('/lobby');
				return;
			}
			const customEvent = event as CustomEvent<{ message?: string }>;
			errorMessage = customEvent.detail?.message ?? 'Battle connection failed';
		};
		const onBattleFinished = () => {
			if (!activeGameId) {
				return;
			}
			battleCompleted = true;

			const phaserGame = game;
			const battleScene = phaserGame?.scene.getScene('BattleScene') as { playerShip?: { dead?: boolean } } | undefined;
			const playerLost = Boolean(battleScene?.playerShip?.dead);
			resultWinner = playerLost
				? battleSetup?.opponents?.[0]?.captainName ?? ''
				: battleSetup?.playerCaptainName ?? '';

			window.setTimeout(() => {
				void completeGame(activeGameId).catch(() => {});
				void goto(`/battle-result?winner=${encodeURIComponent(resultWinner)}`);
			}, 2500);
		};
		const onKeyDown = (event: KeyboardEvent) => {
			if (event.key !== 'Escape' || !activeGameId || battleCompleted) {
				return;
			}
			event.preventDefault();
			if (!window.confirm('Weet je zeker dat je het spel wilt stoppen?')) {
				return;
			}
			void stopBattle();
		};
		window.addEventListener('battlecontrol:scene-ready', onReady, { once: true });
		window.addEventListener('battlecontrol:battle-connection-failed', onConnectionFailed as EventListener);
		window.addEventListener('battlecontrol:battle-finished', onBattleFinished);
		window.addEventListener('keydown', onKeyDown);
		void loadBattle();
		return () => {
			window.removeEventListener('battlecontrol:scene-ready', onReady);
			window.removeEventListener('battlecontrol:battle-connection-failed', onConnectionFailed as EventListener);
			window.removeEventListener('battlecontrol:battle-finished', onBattleFinished);
			window.removeEventListener('keydown', onKeyDown);
		};
	});

	onDestroy(() => {
		game?.destroy(true);
		game = null;
	});

	function startGame() {
		if (started) {
			return;
		}
		started = true;
		triggerBattleStart();
	}

	function triggerBattleStart() {
		if (battleStarted) {
			return;
		}
		battleStarted = true;
		window.dispatchEvent(new Event('battlecontrol:start-game'));
	}

	async function stopBattle() {
		if (!activeGameId) {
			return;
		}
		battleCompleted = true;
		await completeGame(activeGameId);
		window.location.assign('/lobby');
	}

	async function loadBattle() {
		const { createBattleGame } = await import('$lib/game/boot.js');
		let userSettings: UserSettingsDto | null = null;
		try {
			userSettings = await getUserSettings();
		} catch {
			userSettings = null;
		}

		try {
			const gameId = new URL(window.location.href).searchParams.get('gameId');
			if (gameId) {
				activeGameId = gameId;
				started = true;
				const [currentUser, gameInstance] = await Promise.all([
					getCurrentUser(),
					getGameInstance(gameId),
				]);
				const currentPlayer = gameInstance.players.find((player) => player.user.name === currentUser?.name);
				const opponents = gameInstance.players
					.filter((player) => player.user.name !== currentUser?.name)
					.map((player) => ({
						id: player.user.id,
						shipType: player.selected_race ?? 'human-cruiser',
						captainName: player.user.name,
					}));
				battleSetup = {
					playerShipType: currentPlayer?.selected_race ?? 'human-cruiser',
					targetShipType: opponents[0]?.shipType ?? 'human-cruiser',
					playerCaptainName: currentUser?.name ?? currentPlayer?.user.name ?? '',
					gameId,
					opponents,
				};
			}
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Battle setup failed';
			if (activeGameId) {
				return;
			}
		}

		try {
			game = await createBattleGame(gameContainer, hudContainer, userSettings ?? undefined, battleSetup);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Battle startup failed';
		}
	}
</script>

{#if errorMessage}
	<div class="fixed left-1/2 top-4 z-[110] -translate-x-1/2 rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
		{errorMessage}
	</div>
{/if}

{#if !started && !activeGameId}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center
			bg-[radial-gradient(circle_at_center,rgb(10_18_34/55%),rgb(0_0_0/88%))]"
	>
		<button
			class="min-w-[220px] min-h-[64px] px-6 py-3.5 border border-[#8c8c8c]
				bg-gradient-to-b from-[#2f3f56] to-[#162031] text-[#f1f1f1]
				font-mono font-bold text-[22px] uppercase tracking-wider cursor-pointer
				shadow-[0_0_0_1px_rgb(255_255_255/10%)_inset,0_12px_32px_rgb(0_0_0/45%)]
				hover:bg-gradient-to-b hover:from-[#3b4e69] hover:to-[#1b283d]
				disabled:opacity-50 disabled:cursor-default"
			disabled={!sceneReady}
			onclick={startGame}
		>
			{sceneReady ? 'Start Game' : 'Loading...'}
		</button>
	</div>
{/if}

<div class="flex h-screen w-screen">
	<div bind:this={gameContainer} class="flex-1 min-w-0 min-h-0"></div>
	<div bind:this={hudContainer} class="w-[var(--hud-width)] h-screen flex-shrink-0"></div>
</div>
