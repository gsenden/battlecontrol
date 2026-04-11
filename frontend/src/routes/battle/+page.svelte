<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getUserSettings, type UserSettingsDto } from '$lib/auth/auth.js';

	let gameContainer: HTMLDivElement;
	let hudContainer: HTMLDivElement;
	let game: import('phaser').Game | null = null;
	let sceneReady = $state(false);
	let started = $state(false);

	onMount(async () => {
		const { createBattleGame } = await import('$lib/game/boot.js');
		let userSettings: UserSettingsDto | null = null;

		const onReady = () => {
			sceneReady = true;
		};
		window.addEventListener('battlecontrol:scene-ready', onReady, { once: true });

		try {
			userSettings = await getUserSettings();
		} catch {
			userSettings = null;
		}

		game = await createBattleGame(gameContainer, hudContainer, userSettings ?? undefined);
	});

	onDestroy(() => {
		game?.destroy(true);
		game = null;
	});

	function startGame() {
		started = true;
		window.dispatchEvent(new Event('battlecontrol:start-game'));
	}
</script>

{#if !started}
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
