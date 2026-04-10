<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getCurrentUser, registerWithPasskey, toReadableErrorMessage, type UserDto } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	let name = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state('');
	let registeredUser = $state<UserDto | null>(null);

	onMount(() => {
		void loadCurrentUser();
	});

async function loadCurrentUser() {
		try {
			const currentUser = await getCurrentUser();
			if (currentUser) {
				window.location.assign('/lobby');
			}
		} catch {
			// The page should still be usable even if the session probe fails.
		}
	}

	async function submitRegistration() {
		errorMessage = '';
		isSubmitting = true;

		try {
			const trimmedName = name.trim();
			const user = await registerWithPasskey(trimmedName);
			registeredUser = user;
			window.location.assign('/lobby');
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
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
					<AppTitle className="origin-top scale-[0.66] uppercase" title={t('REGISTER', $currentLanguage)} />
				</div>
			</div>
		</div>

		<form class="mx-auto max-w-[560px] space-y-5" onsubmit={(event) => {
			event.preventDefault();
			void submitRegistration();
		}}>
			<div>
				<input
					id="player-name"
					aria-label={t('PLAYER_NAME', $currentLanguage)}
					bind:value={name}
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					maxlength="32"
					placeholder={t('PLAYER_NAME', $currentLanguage)}
					required
				/>
			</div>

			{#if errorMessage}
				<div class="rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
					{errorMessage}
				</div>
			{/if}

			{#if registeredUser}
				<div class="rounded-[12px] border border-[#3f6f52] bg-[#0d1d16] px-4 py-3 text-[14px] text-[#b8f0c7]">
					{t('AUTH_SUCCESS', $currentLanguage)}: {registeredUser.name}
				</div>
			{/if}

			<div class="flex flex-col items-center pt-2">
				<LandingActionButton
					label={t('REGISTER', $currentLanguage)}
					disabled={isSubmitting}
					type="submit"
				/>
				<LandingTextLink
					label={t('CANCEL', $currentLanguage)}
					onclick={() => goto('/')}
				/>
			</div>
		</form>
	</div>
</div>
