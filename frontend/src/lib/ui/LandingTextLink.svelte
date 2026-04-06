<script lang="ts">
	let {
		label,
		onclick
	}: {
		label: string;
		onclick?: () => void;
	} = $props();
</script>

<button
	class="mt-6 text-[18px] text-[#a7bacf] underline underline-offset-4 transition hover:text-[#d6e7ff]"
	{onclick}
	type="button"
>
	{label}
</button>
