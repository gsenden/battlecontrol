<script lang="ts">
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import { getCurrentUser, toReadableErrorMessage, updateUserProfile, uploadProfileImage } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	let name = $state('');
	let profileImageUrl = $state('');
	let errorMessage = $state('');
	let successMessage = $state('');
	let isSubmitting = $state(false);
	let isUploadingImage = $state(false);
	let cropImageUrl = $state('');
	let cropImageWidth = $state(0);
	let cropImageHeight = $state(0);
	let cropScale = $state(1);
	let cropMinScale = $state(1);
	let cropOffsetX = $state(0);
	let cropOffsetY = $state(0);
	let isDraggingCrop = $state(false);
	let dragStartX = 0;
	let dragStartY = 0;
	let dragStartOffsetX = 0;
	let dragStartOffsetY = 0;
	let redirectTimeout: ReturnType<typeof setTimeout> | null = null;
	const CROP_SIZE = 256;

	onMount(() => {
		void loadProfile();
	});

	onDestroy(() => {
		if (redirectTimeout) {
			clearTimeout(redirectTimeout);
		}
		clearCropImage();
	});

	async function loadProfile() {
		try {
			const currentUser = await getCurrentUser();
			if (!currentUser) {
				window.location.assign('/');
				return;
			}

			name = currentUser.name;
			profileImageUrl = currentUser.profile_image_url ?? '';
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	async function saveProfileForm() {
		errorMessage = '';
		successMessage = '';
		isSubmitting = true;

		try {
			let nextProfileImageUrl = profileImageUrl;
			if (cropImageUrl) {
				isUploadingImage = true;
				nextProfileImageUrl = await uploadProfileImage(await renderCroppedImage());
			}

			const updatedUser = await updateUserProfile(name, nextProfileImageUrl);
			profileImageUrl = updatedUser.profile_image_url ?? '';
			clearCropImage();
			window.dispatchEvent(new CustomEvent('battlecontrol:user-updated', { detail: updatedUser }));
			successMessage = t('PROFILE_SAVED', $currentLanguage);
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
			isUploadingImage = false;
		}
	}

	async function handleProfileImageChange(event: Event) {
		const target = event.currentTarget as HTMLInputElement;
		const image = target.files?.[0];
		if (!image) {
			return;
		}

		errorMessage = '';
		successMessage = '';

		try {
			const nextCropImageUrl = URL.createObjectURL(image);
			const dimensions = await loadImageDimensions(nextCropImageUrl);
			clearCropImage();
			cropImageUrl = nextCropImageUrl;
			cropImageWidth = dimensions.width;
			cropImageHeight = dimensions.height;
			cropMinScale = Math.max(CROP_SIZE / dimensions.width, CROP_SIZE / dimensions.height);
			cropScale = cropMinScale;
			cropOffsetX = 0;
			cropOffsetY = 0;
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		}
	}

	function clearCropImage() {
		if (cropImageUrl) {
			URL.revokeObjectURL(cropImageUrl);
		}
		cropImageUrl = '';
		cropImageWidth = 0;
		cropImageHeight = 0;
		cropScale = 1;
		cropMinScale = 1;
		cropOffsetX = 0;
		cropOffsetY = 0;
		isDraggingCrop = false;
	}

	function clampCropOffsets(nextOffsetX: number, nextOffsetY: number) {
		const displayWidth = cropImageWidth * cropScale;
		const displayHeight = cropImageHeight * cropScale;
		const maxOffsetX = Math.max(0, (displayWidth - CROP_SIZE) / 2);
		const maxOffsetY = Math.max(0, (displayHeight - CROP_SIZE) / 2);
		cropOffsetX = Math.min(maxOffsetX, Math.max(-maxOffsetX, nextOffsetX));
		cropOffsetY = Math.min(maxOffsetY, Math.max(-maxOffsetY, nextOffsetY));
	}

	function handleCropScaleChange() {
		clampCropOffsets(cropOffsetX, cropOffsetY);
	}

	function startCropDrag(event: PointerEvent) {
		if (!cropImageUrl) {
			return;
		}

		isDraggingCrop = true;
		dragStartX = event.clientX;
		dragStartY = event.clientY;
		dragStartOffsetX = cropOffsetX;
		dragStartOffsetY = cropOffsetY;
	}

	function handleCropDrag(event: PointerEvent) {
		if (!isDraggingCrop) {
			return;
		}

		clampCropOffsets(
			dragStartOffsetX + (event.clientX - dragStartX),
			dragStartOffsetY + (event.clientY - dragStartY),
		);
	}

	function stopCropDrag() {
		isDraggingCrop = false;
	}

	async function loadImageDimensions(url: string): Promise<{ width: number; height: number }> {
		return await new Promise((resolve, reject) => {
			const image = new Image();
			image.onload = () => resolve({ width: image.naturalWidth, height: image.naturalHeight });
			image.onerror = () => reject(new Error('Could not read image'));
			image.src = url;
		});
	}

	async function renderCroppedImage(): Promise<Blob> {
		const canvas = document.createElement('canvas');
		canvas.width = CROP_SIZE;
		canvas.height = CROP_SIZE;
		const context = canvas.getContext('2d');
		if (!context) {
			throw new Error('Could not prepare image');
		}

		const image = await new Promise<HTMLImageElement>((resolve, reject) => {
			const nextImage = new Image();
			nextImage.onload = () => resolve(nextImage);
			nextImage.onerror = () => reject(new Error('Could not render image'));
			nextImage.src = cropImageUrl;
		});

		const displayWidth = cropImageWidth * cropScale;
		const displayHeight = cropImageHeight * cropScale;
		const imageX = ((CROP_SIZE - displayWidth) / 2) + cropOffsetX;
		const imageY = ((CROP_SIZE - displayHeight) / 2) + cropOffsetY;
		context.drawImage(image, imageX, imageY, displayWidth, displayHeight);

		return await new Promise((resolve, reject) => {
			canvas.toBlob((blob) => {
				if (!blob) {
					reject(new Error('Could not encode image'));
					return;
				}
				resolve(blob);
			}, 'image/webp', 0.9);
		});
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
					<AppTitle className="origin-top scale-[0.66] uppercase" title={t('PROFILE', $currentLanguage)} />
				</div>
			</div>
		</div>

		<form class="mx-auto max-w-[560px] space-y-5" onsubmit={(event) => {
			event.preventDefault();
			void saveProfileForm();
		}}>
			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('PLAYER_NAME', $currentLanguage)}</div>
				<input
					bind:value={name}
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					required
					type="text"
				/>
			</label>

			<label class="block text-[15px] text-[#d9e3ee]">
				<div class="mb-2">{t('PROFILE_IMAGE', $currentLanguage)}</div>
				<input
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					accept="image/*"
					disabled={isUploadingImage}
					onchange={(event) => void handleProfileImageChange(event)}
					type="file"
				/>
			</label>

			{#if cropImageUrl}
				<div class="space-y-4">
					<div
						aria-label={t('PROFILE_IMAGE', $currentLanguage)}
						class="relative mx-auto h-[256px] w-[256px] overflow-hidden rounded-[18px] border border-[#4b5f79] bg-[#08111d] touch-none"
						onpointerdown={startCropDrag}
						onpointermove={handleCropDrag}
						onpointerup={stopCropDrag}
						onpointerleave={stopCropDrag}
						role="img"
					>
						<img
							alt={name}
							class="pointer-events-none absolute left-1/2 top-1/2 max-w-none select-none"
							draggable="false"
							src={cropImageUrl}
							style={`width:${cropImageWidth * cropScale}px;height:${cropImageHeight * cropScale}px;transform:translate(calc(-50% + ${cropOffsetX}px), calc(-50% + ${cropOffsetY}px));`}
						/>
					</div>
					<input
						bind:value={cropScale}
						class="w-full accent-[#83c5ff]"
						max={Math.max(cropMinScale * 3, cropMinScale)}
						min={cropMinScale}
						oninput={handleCropScaleChange}
						step="0.01"
						type="range"
					/>
				</div>
			{:else if profileImageUrl}
				<div class="flex justify-center">
					<img
						alt={name}
						class="h-24 w-24 rounded-full border border-[#4b5f79] object-cover"
						src={profileImageUrl}
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
					label={t('SAVE', $currentLanguage)}
					disabled={isSubmitting || isUploadingImage}
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
