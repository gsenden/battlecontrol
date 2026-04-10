<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getCurrentUser, logoutUser, toReadableErrorMessage, type UserDto } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';

	type LobbyPlayer = {
		id: number;
		name: string;
		status: string;
		isCurrentUser?: boolean;
	};

	type LobbyGame = {
		id: string;
		title: string;
		host: string;
		status: string;
	};

	let currentUser = $state<UserDto | null>(null);
	let errorMessage = $state('');
	let isLoggingOut = $state(false);

	const fallbackPlayers: LobbyPlayer[] = [
		{ id: 101, name: 'Zoq-Fot-Pik Ace', status: 'In lobby' },
		{ id: 102, name: 'Thraddash Burner', status: 'Ready' },
		{ id: 103, name: 'Yehat Guard', status: 'Spectating' }
	];

	const lobbyGames: LobbyGame[] = [
		{ id: 'game-1', title: 'Skirmish over Vela', host: 'Thraddash Burner', status: 'Waiting for opponent' },
		{ id: 'game-2', title: 'Twin Moon Duel', host: 'Yehat Guard', status: 'Filling slots' }
	];

	onMount(() => {
		void loadCurrentUser();
	});

	async function loadCurrentUser() {
		try {
			currentUser = await getCurrentUser();
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	async function signOut() {
		errorMessage = '';
		isLoggingOut = true;

		try {
			await logoutUser();
			await goto('/');
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		} finally {
			isLoggingOut = false;
		}
	}

	function lobbyPlayers(): LobbyPlayer[] {
		if (!currentUser) {
			return fallbackPlayers;
		}

		return [
			{
				id: currentUser.id,
				name: currentUser.name,
				status: t('LOBBY_ONLINE', $currentLanguage),
				isCurrentUser: true
			},
			...fallbackPlayers
		];
	}
</script>

<div class="relative flex h-full flex-col px-6 pb-8 pt-6 text-[#ecf1f7]">
	<div class="mb-8 flex items-start justify-between gap-6">
		<div class="flex items-start gap-6">
			<AppTitle className="origin-top-left scale-[0.25] uppercase" title={t('APP_NAME', $currentLanguage)} />
		</div>

		<button
			class="rounded-[12px] border border-[#7d7d7d] bg-[rgb(15_23_38/28%)] px-5 py-3 text-[16px] font-[700] text-[#f4f8fc] transition hover:border-[#d7d7d7] hover:bg-[rgb(40_52_72/40%)] disabled:opacity-50"
			disabled={isLoggingOut}
			onclick={() => void signOut()}
			type="button"
		>
			{t('LOGOUT', $currentLanguage)}
		</button>
	</div>

	<div class="mb-8 flex justify-center text-center">
		<div class="inline-flex flex-col items-stretch">
			<AppTitle className="origin-top scale-[0.66] uppercase" title={t('LOBBY', $currentLanguage)} />
		</div>
	</div>

	{#if errorMessage}
		<div class="mb-6 max-w-[680px] rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
			{errorMessage}
		</div>
	{/if}

	<div class="mx-auto grid min-h-0 w-fit flex-1 gap-6 lg:grid-cols-2">
		<section class="flex min-h-0 w-full max-w-[440px] flex-col rounded-[20px] border border-[#4b5f79] bg-[rgb(8_17_29/64%)] p-6 shadow-[0_24px_80px_rgb(0_0_0/28%)] backdrop-blur-[2px]">
			<header class="mb-5">
				<h2 class="text-[28px] font-[800] uppercase leading-none text-[#f4f8fc]">{t('LOBBY_PLAYERS', $currentLanguage)}</h2>
			</header>

			<div class="flex min-h-0 flex-1 flex-col gap-3 overflow-auto pr-1">
				{#each lobbyPlayers() as player}
					<div class={`rounded-[14px] border px-4 py-4 transition ${
						player.isCurrentUser
							? 'border-[#8fb9ff] bg-[rgb(46_73_112/28%)] shadow-[0_0_24px_rgb(100_140_210/14%)]'
							: 'border-[#39495f] bg-[rgb(9_15_24/52%)]'
					}`}>
						<div class="flex items-center justify-between gap-4">
							<div class="min-w-0">
								<div class="truncate text-[20px] font-[700] text-[#f4f8fc]">{player.name}</div>
								<div class="mt-1 text-[13px] uppercase tracking-[0.14em] text-[#87a0bf]">{player.status}</div>
							</div>
							{#if player.isCurrentUser}
								<div class="rounded-full border border-[#94bfff] px-3 py-1 text-[11px] font-[800] uppercase tracking-[0.16em] text-[#dbe9ff]">
									{t('LOBBY_ONLINE', $currentLanguage)}
								</div>
							{/if}
						</div>
					</div>
				{:else}
					<div class="rounded-[14px] border border-dashed border-[#435269] bg-[rgb(9_15_24/40%)] px-4 py-6 text-[15px] text-[#96a7bc]">
						{t('LOBBY_EMPTY_PLAYERS', $currentLanguage)}
					</div>
				{/each}
			</div>
		</section>

		<section class="flex min-h-0 w-full max-w-[440px] flex-col rounded-[20px] border border-[#4b5f79] bg-[rgb(8_17_29/64%)] p-6 shadow-[0_24px_80px_rgb(0_0_0/28%)] backdrop-blur-[2px]">
			<header class="mb-5">
				<h2 class="text-[28px] font-[800] uppercase leading-none text-[#f4f8fc]">{t('LOBBY_GAMES', $currentLanguage)}</h2>
			</header>

			<div class="flex min-h-0 flex-1 flex-col gap-3 overflow-auto pr-1">
				{#each lobbyGames as game}
					<div class="rounded-[14px] border border-[#39495f] bg-[rgb(9_15_24/52%)] px-4 py-4">
						<div class="flex items-start justify-between gap-4">
							<div class="min-w-0">
								<div class="truncate text-[20px] font-[700] text-[#f4f8fc]">{game.title}</div>
								<div class="mt-1 text-[14px] text-[#a6b8cd]">{game.host}</div>
							</div>
							<div class="rounded-full border border-[#566a84] px-3 py-1 text-[11px] font-[800] uppercase tracking-[0.16em] text-[#cad8e8]">
								{game.status}
							</div>
						</div>
					</div>
				{:else}
					<div class="rounded-[14px] border border-dashed border-[#435269] bg-[rgb(9_15_24/40%)] px-4 py-6 text-[15px] text-[#96a7bc]">
						{t('LOBBY_EMPTY_GAMES', $currentLanguage)}
					</div>
				{/each}
			</div>
		</section>
	</div>
</div>
