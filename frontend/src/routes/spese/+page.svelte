<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Icon from '#lib/components/Icon.svelte';
	import { api, qs } from '#lib/api.ts';
	import { categoryIcon, categoryLabel, fmtDate, fmtEuro, fmtEuroRound } from '#lib/format.ts';
	import { app, isPending, openDialog } from '#lib/state.svelte.ts';
	import type { Expenses } from '#lib/types.ts';

	let data = $state<Expenses | null>(null);
	let error = $state('');

	const year = $derived(Number(page.url.searchParams.get('anno')) || null);
	const vehicle = $derived(Number(page.url.searchParams.get('auto')) || null);

	$effect(() => {
		void app.version;
		api.get<Expenses>(`/expenses${qs({ year, vehicle_id: vehicle })}`)
			.then((d) => {
				data = d;
				error = '';
			})
			.catch((e) => (error = e.message));
	});

	function setFilter(key: string, value: string | number | null) {
		const url = new URL(page.url.href);
		if (value === null || value === '') url.searchParams.delete(key);
		else url.searchParams.set(key, String(value));
		goto(url, { replace: true, reset: false });
	}

	const maxCat = $derived(Math.max(1, ...(data?.by_category.map((c) => c.total_cents) ?? [1])));
	const maxVeh = $derived(Math.max(1, ...(data?.by_vehicle.map((c) => c.total_cents) ?? [1])));
	const vehicleName = (id: number) => app.vehicles.find((v) => v.id === id)?.name ?? '';
</script>

<svelte:head><title>Spese · MyGarage</title></svelte:head>

<div class="page-head">
	<div>
		<p class="eyebrow">Assicurazione, bollo, manutenzione e il resto</p>
		<h1>Spese</h1>
	</div>
	{#if data}
		<div class="field year">
			<label class="sr-only" for="sp-year">Anno</label>
			<select id="sp-year" class="input" value={data.year} onchange={(e) => setFilter('anno', e.currentTarget.value)}>
				{#each data.years as y (y)}<option value={y}>{y}</option>{/each}
			</select>
		</div>
	{/if}
</div>

{#if app.vehicles.length > 1}
	<div class="filters" role="group" aria-label="Filtra per auto">
		<button class="chip" type="button" aria-pressed={!vehicle} onclick={() => setFilter('auto', null)}>Tutte le auto</button>
		{#each app.vehicles as v (v.id)}
			<button class="chip" type="button" aria-pressed={vehicle === v.id} onclick={() => setFilter('auto', v.id)}>
				<span class="dot" style="background:{v.color}"></span>{v.name}
			</button>
		{/each}
	</div>
{/if}

{#if error}
	<div class="alert danger" role="alert">{error}</div>
{:else if !data}
	<div class="skeleton" style="height:300px"></div>
{:else}
	<div class="grid-2">
		<section class="card card-pad stack" aria-labelledby="h-cat">
			<div class="total">
				<h2 id="h-cat">Totale {data.year}</h2>
				<span class="v num">{fmtEuroRound(data.total_cents)}</span>
			</div>
			{#each data.by_category as c (c.category)}
				<div class="bar-row">
					<span class="lbl"><Icon name={categoryIcon[c.category] ?? 'wrench'} size={16} />{categoryLabel[c.category] ?? c.category}</span>
					<span class="amt num">{fmtEuro(c.total_cents)}</span>
					<div class="bar"><span style="width:{(c.total_cents / maxCat) * 100}%"></span></div>
				</div>
			{:else}
				<p class="subtle" style="margin:0">Nessuna spesa nel {data.year}.</p>
			{/each}
		</section>
		{#if !vehicle && data.by_vehicle.length > 1}
			<section class="card card-pad stack" aria-labelledby="h-veh">
				<h2 id="h-veh">Per auto</h2>
				{#each data.by_vehicle as v (v.vehicle_id)}
					<div class="bar-row">
						<span class="lbl"><span class="dot" style="background:{v.color}"></span>{v.name}</span>
						<span class="amt num">{fmtEuro(v.total_cents)}</span>
						<div class="bar"><span style="width:{(v.total_cents / maxVeh) * 100}%;background:{v.color}"></span></div>
					</div>
				{/each}
			</section>
		{/if}
	</div>

	<section class="section" aria-labelledby="h-list">
		<div class="section-head"><h2 id="h-list">Movimenti</h2></div>
		{#if data.events.length}
			<ul class="card list">
				{#each data.events.filter((e) => !isPending('event', e.id)) as e (e.id)}
					<li>
						<button class="ev" type="button" onclick={() => openDialog({ kind: 'event', vehicleId: e.vehicle_id, event: e })}>
							<span class="tile-ico"><Icon name={categoryIcon[e.category] ?? 'wrench'} size={18} /></span>
							<span class="grow">
								<span class="t">{e.title}</span>
								<span class="s">{fmtDate(e.date)}{vehicle ? '' : ` · ${vehicleName(e.vehicle_id)}`}{e.provider ? ` · ${e.provider}` : ''}</span>
							</span>
							<span class="cost num">{fmtEuro(e.cost_cents)}</span>
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<div class="card empty">
				<span class="tile-ico"><Icon name="wallet" size={26} /></span>
				<p>Le spese sono i costi degli interventi: quando registri un tagliando o paghi l’RCA, indica quanto hai speso.</p>
			</div>
		{/if}
	</section>
{/if}

<style>
	.year {
		min-width: 120px;
	}
	.filters {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 16px;
	}
	.total {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.total .v {
		font-size: 30px;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.bar-row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 6px 12px;
		font-size: 14px;
	}
	.bar-row .lbl {
		font-weight: 600;
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.bar-row .amt {
		font-weight: 700;
	}
	.bar-row .bar {
		grid-column: 1 / -1;
		height: 6px;
	}
	.list li {
		padding: 0;
	}
	.ev {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px;
		border: 0;
		background: transparent;
		border-radius: 12px;
		cursor: pointer;
		text-align: left;
	}
	.ev:hover {
		background: var(--surface-2);
	}
	.ev .grow {
		display: grid;
	}
	.cost {
		font-weight: 700;
	}
</style>
