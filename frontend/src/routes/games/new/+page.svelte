<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getCurrentUser, toReadableErrorMessage } from '$lib/auth/auth.js';
	import { createGame as createGameRequest } from '$lib/games/games.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	const FREE_FOR_ALL = 'free_for_all';
	const FREE_FOR_ALL_MAX_PLAYERS = 16;

	type GameDraft = {
		name: string;
		gameType: string;
		maxPlayers: number;
		isPrivate: boolean;
		password: string;
	};

	let errorMessage = $state('');
	let isSubmitting = $state(false);
	let gameDraft = $state<GameDraft>({
		name: '',
		gameType: FREE_FOR_ALL,
		maxPlayers: 4,
		isPrivate: false,
		password: '',
	});

	onMount(() => {
		void loadCurrentUser();
	});

	async function loadCurrentUser() {
		try {
			const currentUser = await getCurrentUser();
			if (!currentUser) {
				window.location.assign('/');
			}
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	function maxPlayersFor(): number {
		return FREE_FOR_ALL_MAX_PLAYERS;
	}

	async function createGame() {
		errorMessage = '';
		isSubmitting = true;

		try {
			const trimmedName = gameDraft.name.trim();
			if (!trimmedName) {
				errorMessage = t('GAME_NAME_REQUIRED', $currentLanguage);
				return;
			}

			if (gameDraft.isPrivate && !gameDraft.password.trim()) {
				errorMessage = t('GAME_PASSWORD_REQUIRED', $currentLanguage);
				return;
			}

			const createdGame = await createGameRequest({
				name: trimmedName,
				game_type: FREE_FOR_ALL,
				max_players: Math.max(2, Math.min(gameDraft.maxPlayers, maxPlayersFor())),
				is_private: gameDraft.isPrivate,
				password: gameDraft.password.trim() || null,
			});

			await goto(`/briefing-room/${createdGame.id}`);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="relative flex h-full items-start justify-center px-6 pt-[18vh]">
	<div class="absolute left-6 top-6">
		<AppTitle className="origin-top-left scale-[0.25] uppercase" title={t('APP_NAME', $currentLanguage)} />
	</div>

	<div class="w-full max-w-[760px] text-[#ecf1f7]">
		<div class="mb-12 text-center">
			<div class="mx-auto inline-flex flex-col items-stretch">
				<div class="mt-8">
					<AppTitle className="origin-top scale-[0.66] uppercase" title={t('NEW_GAME', $currentLanguage)} />
				</div>
			</div>
		</div>

		<form class="mx-auto max-w-[560px] space-y-5" onsubmit={(event) => {
			event.preventDefault();
			void createGame();
		}}>
			<div>
				<input
					id="game-name"
					aria-label={t('GAME_NAME', $currentLanguage)}
					bind:value={gameDraft.name}
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					maxlength="48"
					placeholder={t('GAME_NAME', $currentLanguage)}
					required
				/>
			</div>

			<div class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('GAME_TYPE', $currentLanguage)}</div>
				<div class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb]">
					{t('FREE_FOR_ALL', $currentLanguage)}
				</div>
			</div>

			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('GAME_MAX_PLAYERS', $currentLanguage)}</div>
				<input
					bind:value={gameDraft.maxPlayers}
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					max={maxPlayersFor()}
					min="2"
					type="number"
				/>
			</label>

			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('GAME_VISIBILITY', $currentLanguage)}</div>
				<select
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					onchange={(event) => {
						const value = (event.currentTarget as HTMLSelectElement).value;
						gameDraft = {
							...gameDraft,
							isPrivate: value === 'private',
							password: value === 'private' ? gameDraft.password : '',
						};
					}}
					value={gameDraft.isPrivate ? 'private' : 'public'}
				>
					<option value="public">{t('PUBLIC', $currentLanguage)}</option>
					<option value="private">{t('PRIVATE', $currentLanguage)}</option>
				</select>
			</label>

			{#if gameDraft.isPrivate}
				<div>
					<input
						id="game-password"
						aria-label={t('GAME_PASSWORD', $currentLanguage)}
						bind:value={gameDraft.password}
						class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
						maxlength="64"
						placeholder={t('GAME_PASSWORD', $currentLanguage)}
						required
						type="password"
					/>
				</div>
			{/if}

			{#if errorMessage}
				<div class="rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
					{errorMessage}
				</div>
			{/if}

			<div class="flex flex-col items-center pt-2">
				<LandingActionButton
					label={t('OK', $currentLanguage)}
					disabled={isSubmitting}
					type="submit"
				/>
				<LandingTextLink
					label={t('CANCEL', $currentLanguage)}
					onclick={() => goto('/lobby')}
				/>
			</div>
		</form>
	</div>
</div>
