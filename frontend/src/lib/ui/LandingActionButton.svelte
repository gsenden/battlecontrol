<script lang="ts">
	let {
		label,
		onclick,
		type = 'button',
		disabled = false,
		className = ''
	}: {
		label: string;
		onclick?: () => void;
		type?: 'button' | 'submit';
		disabled?: boolean;
		className?: string;
	} = $props();
</script>

<button
	class={`min-w-[220px] rounded-[12px] border border-[#7d7d7d] bg-[rgb(15_23_38/28%)] px-8 py-4 text-[28px] font-[800] leading-none text-[#f4f8fc] shadow-[0_0_0_1px_rgb(255_255_255/4%)_inset] transition hover:border-[#d7d7d7] hover:bg-[rgb(40_52_72/40%)] hover:text-white hover:shadow-[0_0_24px_rgb(210_220_235/16%)] disabled:cursor-default disabled:opacity-50 ${className}`}
	{disabled}
	{onclick}
	{type}
>
	{label}
</button>
