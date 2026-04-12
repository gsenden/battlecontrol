<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { getCurrentUser, toReadableErrorMessage } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';

	let errorMessage = $state('');
	const winnerName = $derived(page.url.searchParams.get('winner') ?? '');

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
		<div class="mb-10 text-center">
			<div class="mx-auto inline-flex flex-col items-stretch">
				<div class="mt-8">
					<AppTitle className="origin-top scale-[0.66] uppercase" title={t('BATTLE_VICTORY', $currentLanguage)} />
				</div>
			</div>
		</div>

		{#if errorMessage}
			<div class="mx-auto mb-6 max-w-[560px] rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
				{errorMessage}
			</div>
		{/if}

		<div class="mx-auto max-w-[560px] rounded-[18px] border border-[#465771] bg-[linear-gradient(180deg,rgb(11_19_31/86%),rgb(7_12_22/92%))] px-8 py-8 text-center shadow-[0_24px_80px_rgb(0_0_0/32%)]">
			<div class="mx-auto w-fit rounded-full border border-[#536983] bg-[rgb(18_31_49/72%)] px-4 py-1 text-[12px] font-[700] uppercase tracking-[0.22em] text-[#b5c8df]">
				{t('BATTLE_WINNER', $currentLanguage)}
			</div>

			<div class="mt-6 rounded-[14px] border border-[#314255] bg-[rgb(10_17_28/82%)] px-6 py-7 shadow-[0_0_0_1px_rgb(255_255_255/3%)_inset]">
				<div class="font-[StarCon] text-[18px] uppercase tracking-[0.18em] text-[#90a8c2]">
					{t('BATTLE_VICTORY', $currentLanguage)}
				</div>
				<div class="mt-3 text-[40px] font-[800] leading-none text-[#f4f8fc]">
					{winnerName}
				</div>
			</div>

			<div class="mt-8 flex justify-center">
				<LandingActionButton
					label={t('CLOSE', $currentLanguage)}
					onclick={() => goto('/lobby')}
				/>
			</div>
		</div>
	</div>
</div>
