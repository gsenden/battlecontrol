<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getCurrentUser, toReadableErrorMessage, type UserDto } from '$lib/auth/auth.js';
	import { joinGame, listGames, type GameDto } from '$lib/games/games.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';

	let currentUser = $state<UserDto | null>(null);
	let errorMessage = $state('');
	let lobbyGames = $state<GameDto[]>([]);

	onMount(() => {
		void loadCurrentUser();
	});

	async function loadCurrentUser() {
		try {
			currentUser = await getCurrentUser();
			if (!currentUser) {
				window.location.assign('/');
				return;
			}
			lobbyGames = await listGames();
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	async function openGame(game: GameDto) {
		try {
			const password = game.is_private ? window.prompt(t('GAME_PASSWORD', $currentLanguage)) ?? '' : '';
			await joinGame(game.id, {
				password: game.is_private ? password : null,
			});
			await goto(`/briefing-room/${game.id}`);
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
			<AppTitle className="origin-top scale-[0.66] uppercase" title={t('LOBBY_GAMES', $currentLanguage)} />
		</div>
	</div>

	{#if errorMessage}
		<div class="mb-6 max-w-[680px] rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
			{errorMessage}
		</div>
	{/if}

	<div class="mx-auto flex min-h-0 w-full max-w-[920px] flex-1 justify-center">
		<section class="flex min-h-0 w-full flex-col">
			<div class="mb-5 flex justify-end">
				<button
					aria-label={t('NEW_GAME', $currentLanguage)}
					class="inline-flex h-9 w-12 items-center justify-center rounded-[12px] border border-[#3f8f63] bg-[#102819] text-[#d9ffe7] shadow-[0_10px_24px_rgb(10_32_18/22%)] transition hover:border-[#6fe49c] hover:bg-[#173922] hover:text-white active:translate-y-[1px] active:border-[#2f6c4a] active:bg-[#0c1e13] active:shadow-[0_4px_10px_rgb(10_32_18/18%)]"
					onclick={() => goto('/games/new')}
					type="button"
				>
					<svg aria-hidden="true" class="h-4 w-4" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" viewBox="0 0 24 24">
						<path d="M12 5v14" />
						<path d="M5 12h14" />
					</svg>
				</button>
			</div>
			<div class="min-h-0 overflow-auto">
				<table class="w-full border-separate border-spacing-y-3 text-left">
					<thead>
						<tr class="text-[14px] uppercase tracking-[0.16em] text-[#9cb0c8]">
							<th class="px-4 pb-1 font-[800]">{t('GAME_NAME', $currentLanguage)}</th>
							<th class="px-4 pb-1 font-[800]">{t('GAME_TYPE', $currentLanguage)}</th>
							<th class="px-4 pb-1 font-[800]">{t('GAME_CREATOR', $currentLanguage)}</th>
							<th class="px-4 pb-1 font-[800]">{t('GAME_PLAYER_COUNT', $currentLanguage)}</th>
							<th class="px-4 pb-1 font-[800]">{t('GAME_PRIVATE', $currentLanguage)}</th>
						</tr>
					</thead>
					<tbody>
				{#each lobbyGames as game}
					<tr class="cursor-pointer rounded-[14px] border border-[#39495f] bg-[rgb(9_15_24/52%)] transition hover:bg-[rgb(13_21_33/68%)]" onclick={() => void openGame(game)}>
						<td class="rounded-l-[14px] border-y border-l border-[#39495f] px-4 py-4 text-[20px] font-[700] text-[#f4f8fc]">{game.name}</td>
						<td class="border-y border-[#39495f] px-4 py-4 text-[16px] text-[#d9e3ef]">
							{game.game_type === 'teams' ? t('TEAMS', $currentLanguage) : t('FREE_FOR_ALL', $currentLanguage)}
						</td>
						<td class="border-y border-[#39495f] px-4 py-4 text-[16px] text-[#a6b8cd]">
							<div class="flex items-center gap-3">
								{#if game.creator.profile_image_url}
									<img
										alt={game.creator.name}
										class="h-8 w-8 rounded-full border border-[#435269] object-cover"
										src={game.creator.profile_image_url}
									/>
								{:else}
									<div class="flex h-8 w-8 items-center justify-center rounded-full border border-[#435269] bg-[#101923] text-[#d8e1ec]">
										<svg aria-hidden="true" class="h-4 w-4" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
											<path d="M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z" />
											<path d="M4 20a8 8 0 0 1 16 0" />
										</svg>
									</div>
								{/if}
								<span>{game.creator.name}</span>
							</div>
						</td>
						<td class="border-y border-[#39495f] px-4 py-4 text-[16px] text-[#d9e3ef]">{game.players.length}/{game.max_players}</td>
						<td class="rounded-r-[14px] border-y border-r border-[#39495f] px-4 py-4 text-[16px] text-[#d9e3ef]">
							{game.is_private ? t('YES', $currentLanguage) : t('NO', $currentLanguage)}
						</td>
					</tr>
				{:else}
					<tr>
						<td colspan="5" class="rounded-[14px] border border-dashed border-[#435269] bg-[rgb(9_15_24/40%)] px-4 py-6 text-[15px] text-[#96a7bc]">
							{t('LOBBY_EMPTY_GAMES', $currentLanguage)}
						</td>
					</tr>
				{/each}
					</tbody>
				</table>
			</div>
		</section>
	</div>
</div>
