<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { loadStoredUser, loginWithPasskey, storeUser, toReadableErrorMessage, type UserDto } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';

	let name = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state('');
	let loggedInUser = $state<UserDto | null>(null);

	onMount(() => {
		const storedUser = loadStoredUser();
		if (storedUser) {
			name = storedUser.name;
		}
	});

	async function submitLogin() {
		errorMessage = '';
		isSubmitting = true;

		try {
			const user = await loginWithPasskey(name.trim());
			storeUser(user);
			loggedInUser = user;
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="flex h-full items-center justify-center px-6">
	<div class="w-full max-w-[560px] rounded-[20px] border border-[#47607a] bg-grey p-8 text-[#ecf1f7] shadow-[0_28px_80px_rgb(0_0_0/45%)] backdrop-blur-[2px]">
		<div class="mb-8">
			<AppTitle className="mt-3 uppercase" title={t('APP_NAME', $currentLanguage)} />
			<p class="mt-3 text-[15px] leading-6 text-[#a7bacf]">{t('LOGIN_SUBTITLE', $currentLanguage)}</p>
		</div>

		<form class="space-y-5" onsubmit={(event) => {
			event.preventDefault();
			void submitLogin();
		}}>
			<div class="space-y-2">
				<label class="block font-mono text-[12px] uppercase tracking-[0.24em] text-[#86a8cc]" for="player-name">
					{t('PLAYER_NAME', $currentLanguage)}
				</label>
				<input
					id="player-name"
					bind:value={name}
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					required
				/>
			</div>

			{#if errorMessage}
				<div class="rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
					{errorMessage}
				</div>
			{/if}

			{#if loggedInUser}
				<div class="rounded-[12px] border border-[#3f6f52] bg-[#0d1d16] px-4 py-3 text-[14px] text-[#b8f0c7]">
					{t('AUTH_SUCCESS', $currentLanguage)}: {loggedInUser.name}
				</div>
			{/if}

			<div class="flex flex-wrap gap-3 pt-2">
				<button
					class="min-w-[220px] min-h-[56px] rounded-[12px] border border-[#7aa4d6] bg-[linear-gradient(180deg,#35567c_0%,#1b3048_100%)] px-6 py-3 font-mono text-[18px] font-bold uppercase tracking-[0.12em] text-[#f4f8fc] shadow-[0_0_0_1px_rgb(255_255_255/10%)_inset] transition hover:border-[#9bc6ff] disabled:cursor-default disabled:opacity-60"
					disabled={isSubmitting}
					type="submit"
				>
					{t('LOGIN', $currentLanguage)}
				</button>

				<button
					class="min-w-[220px] min-h-[56px] rounded-[12px] border border-[#7b8ca3] bg-[linear-gradient(180deg,#263448_0%,#111a27_100%)] px-6 py-3 font-mono text-[18px] font-bold uppercase tracking-[0.12em] text-[#f4f8fc] shadow-[0_0_0_1px_rgb(255_255_255/8%)_inset] transition hover:border-[#a6b5ca]"
					onclick={() => goto('/')}
					type="button"
				>
					{t('SIGN_UP', $currentLanguage)}
				</button>

				<button
					class="min-w-[220px] min-h-[56px] rounded-[12px] border border-[#7b8ca3] bg-[linear-gradient(180deg,#263448_0%,#111a27_100%)] px-6 py-3 font-mono text-[18px] font-bold uppercase tracking-[0.12em] text-[#f4f8fc] shadow-[0_0_0_1px_rgb(255_255_255/8%)_inset] transition hover:border-[#a6b5ca] disabled:cursor-default disabled:opacity-50"
					disabled={!loggedInUser}
					onclick={() => goto('/battle')}
					type="button"
				>
					{t('START_BATTLE', $currentLanguage)}
				</button>
			</div>
		</form>
	</div>
</div>
