<script lang="ts">
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import { getCurrentUser, getUserSettings, saveUserSettings, toReadableErrorMessage, type UserSettingsDto } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	let settings = $state<UserSettingsDto>({
		turn_left_key: 'A',
		turn_right_key: 'D',
		thrust_key: 'W',
		music_enabled: true,
		music_volume: 45,
		sound_effects_enabled: true,
		sound_effects_volume: 60,
	});
	let errorMessage = $state('');
	let successMessage = $state('');
	let isSubmitting = $state(false);
	let redirectTimeout: ReturnType<typeof setTimeout> | null = null;

	onMount(() => {
		void loadSettings();
	});

	onDestroy(() => {
		if (redirectTimeout) {
			clearTimeout(redirectTimeout);
		}
	});

	async function loadSettings() {
		try {
			const currentUser = await getCurrentUser();
			if (!currentUser) {
				window.location.assign('/');
				return;
			}

			settings = await getUserSettings();
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	async function saveSettingsForm() {
		errorMessage = '';
		successMessage = '';
		isSubmitting = true;

		try {
			settings = await saveUserSettings(settings);
			successMessage = t('SETTINGS_SAVED', $currentLanguage);
			if (redirectTimeout) {
				clearTimeout(redirectTimeout);
			}
			redirectTimeout = setTimeout(() => {
				void goto('/menu');
			}, 900);
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function handleKeyCapture(event: KeyboardEvent, settingKey: keyof UserSettingsDto) {
		event.preventDefault();
		errorMessage = '';
		successMessage = '';
		settings = {
			...settings,
			[settingKey]: normalizeKey(event.key),
		};
	}

	function normalizeKey(key: string): string {
		const normalizedKey = key.trim().toUpperCase();

		if (normalizedKey === ' ') {
			return 'SPACE';
		}

		if (normalizedKey === 'ARROWLEFT') {
			return 'LEFT';
		}

		if (normalizedKey === 'ARROWRIGHT') {
			return 'RIGHT';
		}

		if (normalizedKey === 'ARROWUP') {
			return 'UP';
		}

		if (normalizedKey === 'ARROWDOWN') {
			return 'DOWN';
		}

		return normalizedKey;
	}

	function toggleSetting(settingKey: 'music_enabled' | 'sound_effects_enabled') {
		errorMessage = '';
		successMessage = '';
		settings = {
			...settings,
			[settingKey]: !settings[settingKey],
		};
	}
</script>

{#if successMessage}
	<div class="fixed bottom-6 left-1/2 z-[120] -translate-x-1/2 rounded-[12px] border border-[#3f6f52] bg-[#0d1d16f2] px-5 py-3 text-[14px] text-[#b8f0c7] shadow-[0_16px_32px_rgb(0_0_0/40%)]">
		{successMessage}
	</div>
{/if}

<div class="relative flex h-full items-start justify-center px-6 pt-[18vh]">
	<div class="absolute left-6 top-6">
		<AppTitle className="origin-top-left scale-[0.25] uppercase" title={t('APP_NAME', $currentLanguage)} />
	</div>

	<div class="w-full max-w-[760px] text-[#ecf1f7]">
		<div class="mb-12 text-center">
			<div class="mx-auto inline-flex flex-col items-stretch">
				<div class="mt-8">
					<AppTitle className="origin-top scale-[0.66] uppercase" title={t('SETTINGS', $currentLanguage)} />
				</div>
			</div>
		</div>

		<form class="mx-auto max-w-[560px] space-y-5" onsubmit={(event) => {
			event.preventDefault();
			void saveSettingsForm();
		}}>
			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('TURN_LEFT', $currentLanguage)}</div>
				<input
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					onkeydown={(event) => handleKeyCapture(event, 'turn_left_key')}
					readonly
					value={settings.turn_left_key}
				/>
			</label>

			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('THRUST', $currentLanguage)}</div>
				<input
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					onkeydown={(event) => handleKeyCapture(event, 'thrust_key')}
					readonly
					value={settings.thrust_key}
				/>
			</label>

			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('TURN_RIGHT', $currentLanguage)}</div>
				<input
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					onkeydown={(event) => handleKeyCapture(event, 'turn_right_key')}
					readonly
					value={settings.turn_right_key}
				/>
			</label>

			<div class="rounded-[14px] border border-[#314559] bg-[#08111dcc] px-4 py-4">
				<div class="mb-3 text-[15px] text-[#d9e3ee]">{t('MUSIC', $currentLanguage)}</div>
				<div class="flex items-center gap-4">
					<input
						bind:value={settings.music_volume}
						class="w-full accent-[#83c5ff]"
						disabled={!settings.music_enabled}
						max="100"
						min="0"
						type="range"
					/>
					<div class="w-[3ch] text-right text-[14px] text-[#c8d7e6]">{settings.music_volume}</div>
					<button
						aria-label={settings.music_enabled ? t('DISABLE_MUSIC', $currentLanguage) : t('ENABLE_MUSIC', $currentLanguage)}
						class="flex h-11 w-11 items-center justify-center rounded-[12px] border border-[#3d5570] bg-[#08111d] text-[#d9e3ee] transition hover:border-[#83c5ff] hover:text-[#f3f7fb]"
						onclick={() => toggleSetting('music_enabled')}
						type="button"
					>
						{#if settings.music_enabled}
							<svg aria-hidden="true" class="h-5 w-5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
								<path d="M11 5 6 9H3v6h3l5 4V5Z" />
								<path d="M15 9a5 5 0 0 1 0 6" />
								<path d="M18.5 6.5a9 9 0 0 1 0 11" />
							</svg>
						{:else}
							<svg aria-hidden="true" class="h-5 w-5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
								<path d="M11 5 6 9H3v6h3l5 4V5Z" />
								<path d="M4 4 20 20" />
							</svg>
						{/if}
					</button>
				</div>
			</div>

			<div class="rounded-[14px] border border-[#314559] bg-[#08111dcc] px-4 py-4">
				<div class="mb-3 text-[15px] text-[#d9e3ee]">{t('SOUND_EFFECTS', $currentLanguage)}</div>
				<div class="flex items-center gap-4">
					<input
						bind:value={settings.sound_effects_volume}
						class="w-full accent-[#83c5ff]"
						disabled={!settings.sound_effects_enabled}
						max="100"
						min="0"
						type="range"
					/>
					<div class="w-[3ch] text-right text-[14px] text-[#c8d7e6]">{settings.sound_effects_volume}</div>
					<button
						aria-label={settings.sound_effects_enabled ? t('DISABLE_SOUND_EFFECTS', $currentLanguage) : t('ENABLE_SOUND_EFFECTS', $currentLanguage)}
						class="flex h-11 w-11 items-center justify-center rounded-[12px] border border-[#3d5570] bg-[#08111d] text-[#d9e3ee] transition hover:border-[#83c5ff] hover:text-[#f3f7fb]"
						onclick={() => toggleSetting('sound_effects_enabled')}
						type="button"
					>
						{#if settings.sound_effects_enabled}
							<svg aria-hidden="true" class="h-5 w-5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
								<path d="M11 5 6 9H3v6h3l5 4V5Z" />
								<path d="M15 9a5 5 0 0 1 0 6" />
								<path d="M18.5 6.5a9 9 0 0 1 0 11" />
							</svg>
						{:else}
							<svg aria-hidden="true" class="h-5 w-5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
								<path d="M11 5 6 9H3v6h3l5 4V5Z" />
								<path d="M4 4 20 20" />
							</svg>
						{/if}
					</button>
				</div>
			</div>

			{#if errorMessage}
				<div class="rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
					{errorMessage}
				</div>
			{/if}

			<div class="flex flex-col items-center pt-2">
				<LandingActionButton
					label={t('SAVE', $currentLanguage)}
					disabled={isSubmitting}
					type="submit"
				/>
				<LandingTextLink
					label={t('CANCEL', $currentLanguage)}
					onclick={() => goto('/menu')}
				/>
			</div>
		</form>
	</div>
</div>
