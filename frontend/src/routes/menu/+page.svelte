<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getCurrentUser, toReadableErrorMessage } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';

	let errorMessage = $state('');

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
</script>

<div class="relative flex h-full items-start justify-center px-6 pt-[18vh]">
	<div class="absolute left-6 top-6">
		<AppTitle className="origin-top-left scale-[0.25] uppercase" title={t('APP_NAME', $currentLanguage)} />
	</div>

	<div class="w-full max-w-[760px] text-[#ecf1f7]">
		<div class="mb-12 text-center">
			<div class="mx-auto inline-flex flex-col items-stretch">
				<div class="mt-8">
					<AppTitle className="origin-top scale-[0.66] uppercase" title={t('APP_NAME', $currentLanguage)} />
				</div>
			</div>
		</div>

		{#if errorMessage}
			<div class="mx-auto mb-6 max-w-[560px] rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
				{errorMessage}
			</div>
		{/if}

		<div class="flex flex-col items-center gap-8 pt-6">
			<LandingActionButton
				label={t('SETTINGS', $currentLanguage)}
				onclick={() => goto('/settings')}
			/>
			<LandingActionButton
				label={t('START_GAME', $currentLanguage)}
				onclick={() => goto('/lobby')}
			/>
		</div>
	</div>
</div>
