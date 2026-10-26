<script lang="ts">
	import type { DueMode } from '#lib/types.ts';
	import { todayIso } from '#lib/format.ts';

	let {
		mode = $bindable(),
		date = $bindable(),
		km = $bindable(),
		allowKeep = false,
		usesDate = true,
		usesKm = false,
		annual = false,
		compact = false
	}: {
		mode: DueMode;
		date: string;
		km: string;
		allowKeep?: boolean;
		usesDate?: boolean;
		usesKm?: boolean;
		annual?: boolean;
		compact?: boolean;
	} = $props();

	const uid = Math.random().toString(36).slice(2);
	const showDate = $derived(usesDate || mode === 'next');
	const showKm = $derived(usesKm);
</script>

<div class="field">
	{#if !compact}<span class="label">Situazione attuale</span>{/if}
	<div class="segmented" role="group" aria-label="Situazione attuale">
		{#if allowKeep}<button type="button" aria-pressed={mode === 'keep'} onclick={() => (mode = 'keep')}>Invariata</button>{/if}
		<button type="button" aria-pressed={mode === 'last'} onclick={() => (mode = 'last')}>Ultima volta</button>
		<button type="button" aria-pressed={mode === 'next'} onclick={() => (mode = 'next')}>Prossima</button>
		<button type="button" aria-pressed={mode === 'unknown'} onclick={() => (mode = 'unknown')}>{annual ? 'Automatica' : 'Non lo so'}</button>
	</div>
	{#if mode === 'unknown'}
		<span class="hint">
			{annual
				? 'La scadenza viene calcolata dalla data fissa.'
				: 'La scadenza resterà “da completare” finché non aggiungi una data o un intervento.'}
		</span>
	{:else if mode === 'keep'}
		<span class="hint">La scadenza attuale non cambia.</span>
	{/if}
</div>

{#if mode === 'last' || mode === 'next'}
	<div class="form-grid cols-2">
		{#if showDate}
			<div class="field">
				<label for="ds-date-{uid}">{mode === 'last' ? 'Fatta il' : 'Scade il'}</label>
				<input id="ds-date-{uid}" class="input" type="date" bind:value={date} max={mode === 'last' ? todayIso() : undefined} />
			</div>
		{/if}
		{#if showKm}
			<div class="field">
				<label for="ds-km-{uid}">{mode === 'last' ? 'A quanti km' : 'Scade a (km)'}</label>
				<div class="input-affix">
					<input id="ds-km-{uid}" class="input num" type="text" inputmode="numeric" bind:value={km} />
					<span class="affix">km</span>
				</div>
			</div>
		{/if}
	</div>
{/if}
