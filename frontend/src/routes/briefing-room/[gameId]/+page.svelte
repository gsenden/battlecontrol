<script lang="ts">
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import { getCurrentUser, toReadableErrorMessage, type UserDto } from '$lib/auth/auth.js';
	import { cancelGame, getGame, leaveGame, saveSelectedRace, startGame, type GameDto, type GameRoomEventDto } from '$lib/games/games.js';
	import { initGameLogic } from '$lib/game/game-logic.js';
	import { buildShipPresets, type ShipPreset } from '$lib/game/ships/ship-presets.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	const { data } = $props<{ data: { gameId: string } }>();
	const AVAILABLE_SHIP_PREFIXES = ['human-cruiser', 'arilou-skiff', 'androsynth-guardian'];

	let currentUser = $state<UserDto | null>(null);
	let game = $state<GameDto | null>(null);
	let errorMessage = $state('');
	let selectedRace = $state('');
	let shipPresets = $state<ShipPreset[]>([]);
	let raceMenuOpen = $state(false);
	let gameSocket: WebSocket | null = null;

	onMount(() => {
		void loadBriefingRoom();
	});

	onDestroy(() => {
		gameSocket?.close();
	});

	async function loadBriefingRoom() {
		try {
			currentUser = await getCurrentUser();
			if (!currentUser) {
				window.location.assign('/');
				return;
			}

			game = await getGame(data.gameId);
			shipPresets = buildShipPresets(await initGameLogic()).filter((preset) =>
				AVAILABLE_SHIP_PREFIXES.includes(preset.stats.spritePrefix),
			);
			selectedRace = game.players.find((player) => player.user.name === currentUser.name)?.selected_race
				?? shipPresets[0]?.stats.spritePrefix
				?? '';
			connectGameRoom();
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	function connectGameRoom() {
		gameSocket?.close();
		const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		gameSocket = new WebSocket(`${protocol}//${window.location.host}/games/${data.gameId}/events`);
		gameSocket.onmessage = (event) => {
			try {
				handleGameRoomEvent(JSON.parse(event.data) as GameRoomEventDto);
			} catch (error) {
				errorMessage = toReadableErrorMessage(error);
			}
		};
	}

	function handleGameRoomEvent(event: GameRoomEventDto) {
		if (event.kind === 'cancelled') {
			window.location.assign('/lobby');
			return;
		}

		if (event.kind === 'started') {
			window.location.assign(`/battle?gameId=${event.game_id}`);
			return;
		}

		if (event.game) {
			game = event.game;
			const currentPlayer = event.game.players.find((player) => player.user.name === currentUser?.name);
			if (currentPlayer?.selected_race) {
				selectedRace = currentPlayer.selected_race;
			}
		}
	}

	function waitingSlots(): number[] {
		if (!game) {
			return [];
		}

		return Array.from({ length: Math.max(game.max_players - game.players.length, 0) }, (_, index) => index + 1);
	}

	function selectedRacePreset(): ShipPreset | null {
		return shipPresets.find((preset) => preset.stats.spritePrefix === selectedRace) ?? null;
	}

	async function chooseRace(spritePrefix: string) {
		selectedRace = spritePrefix;
		raceMenuOpen = false;
		try {
			game = await saveSelectedRace(data.gameId, {
				selected_race: spritePrefix,
			});
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	function racePresetFor(spritePrefix?: string | null): ShipPreset | null {
		if (!spritePrefix) {
			return null;
		}

		return shipPresets.find((preset) => preset.stats.spritePrefix === spritePrefix) ?? null;
	}

	function currentUserIsHost(): boolean {
		return game?.creator.name === currentUser?.name;
	}

	async function cancelOrLeaveGame() {
		try {
			if (currentUserIsHost()) {
				await cancelGame(data.gameId);
			} else {
				await leaveGame(data.gameId);
			}
			window.location.assign('/lobby');
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	async function startCurrentGame() {
		try {
			await startGame(data.gameId);
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}
</script>

<div class="relative flex h-full flex-col px-6 pb-8 pt-6 text-[#ecf1f7]">
	<div class="mb-8 flex items-start justify-between gap-6">
		<div class="flex items-start gap-6">
			<AppTitle className="origin-top-left scale-[0.25] uppercase" title={t('APP_NAME', $currentLanguage)} />
		</div>
	</div>

	<div class="mb-8 flex justify-center text-center">
		<div class="inline-flex flex-col items-stretch">
			<AppTitle className="origin-top scale-[0.66] uppercase" title={t('BRIEFING_ROOM', $currentLanguage)} />
		</div>
	</div>

	{#if errorMessage}
		<div class="mx-auto mb-6 w-full max-w-[720px] rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
			{errorMessage}
		</div>
	{/if}

	{#if game}
		<div class="mx-auto flex w-full max-w-[920px] flex-1 flex-col gap-8">
			<div class="rounded-[16px] border border-[#39495f] bg-[rgb(9_15_24/52%)] px-5 py-4">
				<div class="flex items-center justify-between gap-4">
					<div>
						<div class="text-[24px] font-[800] text-[#f4f8fc]">
							{game.name}
						</div>
						<div class="mt-2 flex flex-wrap gap-6 text-[14px] uppercase tracking-[0.14em] text-[#9cb0c8]">
							<div>
								{t('GAME_TYPE', $currentLanguage)}: {game.game_type === 'teams'
									? t('TEAMS', $currentLanguage)
									: t('FREE_FOR_ALL', $currentLanguage)}
							</div>
							<div>
								{t('GAME_PLAYER_COUNT', $currentLanguage)}: {game.players.length}/{game.max_players}
							</div>
							{#if game.is_private && game.password}
								<div>
									{t('GAME_PASSWORD', $currentLanguage)}: {game.password}
								</div>
							{/if}
						</div>
					</div>
					<div class="flex flex-col items-center justify-center">
						{#if currentUserIsHost()}
							<button
								aria-label={t('START_GAME', $currentLanguage)}
								class="inline-flex h-11 w-11 items-center justify-center rounded-full border border-[#3f8f63] bg-[#102819] text-[#d9ffe7] shadow-[0_10px_24px_rgb(10_32_18/22%)] transition hover:border-[#6fe49c] hover:bg-[#173922] hover:text-white active:border-[#2f6c4a] active:bg-[#0c1e13] active:shadow-[0_4px_10px_rgb(10_32_18/18%)]"
								onclick={() => void startCurrentGame()}
								type="button"
							>
								<svg aria-hidden="true" class="h-4 w-4 translate-x-[1px]" fill="currentColor" viewBox="0 0 24 24">
									<path d="M8 6v12l10-6-10-6Z" />
								</svg>
							</button>
						{/if}
						
					</div>
				</div>
			</div>

			<div class="space-y-3">
				<div class="grid gap-4 border-b border-[#39495f] px-2 pb-2 text-[14px] uppercase tracking-[0.16em] text-[#9cb0c8] md:grid-cols-[1fr_auto] md:items-center">
					<div>{t('PLAYERS', $currentLanguage)}</div>
					<div>{t('RACE', $currentLanguage)}</div>
				</div>

				{#each game.players as player}
					<div class="border-b border-[#39495f] px-2 py-4">
						<div class="grid gap-4 md:grid-cols-[1fr_auto] md:items-center">
							<div class="flex items-center gap-4">
								{#if player.user.profile_image_url}
									<img alt={player.user.name} class="h-12 w-12 rounded-full border border-[#5a7394] object-cover" src={player.user.profile_image_url} />
								{:else}
									<div class="flex h-12 w-12 items-center justify-center rounded-full border border-[#5a7394] bg-[#101923] text-[#d8e1ec]">
										<svg aria-hidden="true" class="h-6 w-6" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
											<path d="M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" />
											<path d="M4 20a8 8 0 0 1 16 0" />
										</svg>
									</div>
								{/if}
								<div>
									<div class="text-[18px] font-[700] text-[#f4f8fc]">{player.user.name}</div>
									<div class="mt-1 text-[13px] uppercase tracking-[0.14em] text-[#9cb0c8]">
										{player.user.name === game.creator.name ? t('HOST', $currentLanguage) : ''}
									</div>
								</div>
							</div>

							{#if player.user.name === currentUser?.name}
								<div class="flex items-center gap-3 px-1 py-2">
									{#if selectedRacePreset()}
										<img
											alt={selectedRacePreset()?.stats.raceName}
											class="h-10 w-10 rounded-[8px] border border-[#435269] bg-[#101923] p-1 object-contain [image-rendering:pixelated]"
											src={selectedRacePreset()?.selectionPortraitUrl}
										/>
									{/if}
									<div class="relative min-w-[220px]">
										<button
											aria-expanded={raceMenuOpen}
											class="flex w-full items-center justify-between gap-3 rounded-[10px] border border-[#435269] bg-[#08111d] px-3 py-2 text-left text-[15px] font-[700] text-[#f4f8fc]"
											onclick={() => (raceMenuOpen = !raceMenuOpen)}
											type="button"
										>
											<span class="truncate">{selectedRacePreset()?.stats.raceName}</span>
											<svg aria-hidden="true" class="h-4 w-4 shrink-0" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
												<path d="m6 9 6 6 6-6" />
											</svg>
										</button>

										{#if raceMenuOpen}
											<div class="absolute right-0 z-20 mt-2 max-h-[320px] w-full overflow-y-auto rounded-[12px] border border-[#435269] bg-[#08111d] p-2 shadow-[0_18px_50px_rgb(2_6_12/42%)]">
												{#each shipPresets as preset}
													<button
														class="flex w-full items-center gap-3 rounded-[10px] px-2 py-2 text-left text-[#eaf1f8] transition hover:bg-[#112035]"
														onclick={() => void chooseRace(preset.stats.spritePrefix)}
														type="button"
													>
														<img alt={preset.stats.raceName} class="h-10 w-10 rounded-[8px] bg-[#101923] p-1 object-contain [image-rendering:pixelated]" src={preset.selectionPortraitUrl} />
														<span class="text-[15px] font-[700]">{preset.stats.raceName}</span>
													</button>
												{/each}
											</div>
										{/if}
									</div>
								</div>
							{:else}
								<div class="flex items-center gap-3 px-1 py-2 text-[15px] text-[#9cb0c8]">
									{#if racePresetFor(player.selected_race)}
										<img
											alt={racePresetFor(player.selected_race)?.stats.raceName}
											class="h-10 w-10 rounded-[8px] border border-[#435269] bg-[#101923] p-1 object-contain [image-rendering:pixelated]"
											src={racePresetFor(player.selected_race)?.selectionPortraitUrl}
										/>
										<span>{racePresetFor(player.selected_race)?.stats.raceName}</span>
									{:else}
										<div class="flex h-10 w-10 items-center justify-center rounded-[8px] border border-dashed border-[#435269] bg-[#101923]">
											<svg aria-hidden="true" class="h-5 w-5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
												<path d="M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" />
												<path d="M4 20a8 8 0 0 1 16 0" />
											</svg>
										</div>
										<span>{t('NO_RACE_SELECTED', $currentLanguage)}</span>
									{/if}
								</div>
							{/if}
						</div>
					</div>
				{/each}

				{#each waitingSlots() as slot}
					<div class="border-b border-[#39495f] px-2 py-4">
						<div class="grid gap-4 md:grid-cols-[1fr_auto] md:items-center">
							<div class="flex items-center gap-4">
								<div class="flex h-12 w-12 items-center justify-center rounded-full border border-dashed border-[#5a7394] bg-[#101923] text-[#7f93aa]">
									<svg aria-hidden="true" class="h-6 w-6" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
										<path d="M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" />
										<path d="M4 20a8 8 0 0 1 16 0" />
									</svg>
								</div>
								<div>
									<div class="text-[18px] font-[700] text-[#9cb0c8]">
										{t('PLAYER', $currentLanguage)} {slot + game.players.length}
									</div>
									<div class="mt-1 text-[13px] uppercase tracking-[0.14em] text-[#6f839b]">
										{t('WAITING', $currentLanguage)}
									</div>
								</div>
							</div>
							<div class="flex items-center gap-3 px-1 py-2 text-[15px] text-[#9cb0c8]">
								<div class="flex h-10 w-10 items-center justify-center rounded-[8px] border border-dashed border-[#435269] bg-[#101923]">
									<svg aria-hidden="true" class="h-5 w-5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
										<path d="M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" />
										<path d="M4 20a8 8 0 0 1 16 0" />
									</svg>
								</div>
								<span>{t('NO_RACE_SELECTED', $currentLanguage)}</span>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
	<LandingTextLink className={currentUserIsHost() ? 'mt-2' : 'mt-0'} label={t('CANCEL', $currentLanguage)} onclick={() => void cancelOrLeaveGame()} />
</div>
