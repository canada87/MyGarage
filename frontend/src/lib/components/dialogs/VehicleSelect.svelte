<script lang="ts">
	import { app } from '#lib/state.svelte.ts';

	let { value = $bindable(), label = 'Auto', allowNone = false }: { value: number | null; label?: string; allowNone?: boolean } = $props();
	const id = `veh-${Math.random().toString(36).slice(2)}`;
</script>

<div class="field">
	<label for={id}>{label}</label>
	<select {id} class="input" bind:value>
		{#if allowNone}<option value={null}>Tutte le auto (generale)</option>{/if}
		{#each app.vehicles as v (v.id)}
			<option value={v.id}>{v.name}{v.plate ? ` · ${v.plate}` : ''}</option>
		{/each}
	</select>
</div>
