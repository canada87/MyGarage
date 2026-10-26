<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';
	import DueRow from '#lib/components/DueRow.svelte';
	import VehicleCard from '#lib/components/VehicleCard.svelte';
	import { api } from '#lib/api.ts';
	import { categoryIcon, categoryLabel, fmtDateLong, fmtEuroRound } from '#lib/format.ts';
	import { app } from '#lib/state.svelte.ts';
	import type { Dashboard } from '#lib/types.ts';

	let data = $state<Dashboard | null>(null);
	let error = $state('');

	$effect(() => {
		void app.version;
		api.get<Dashboard>('/dashboard')
			.then((d) => {
				data = d;
				error = '';
			})
			.catch((e) => (error = e.message));
	});

	const maxCat = $derived(Math.max(1, ...(data?.year_by_category.map((c) => c.total_cents) ?? [1])));
</script>

<svelte:head><title>MyGarage</title></svelte:head>

<div class="page-head">
	<div>
		<p class="eyebrow">{data ? fmtDateLong(data.today) : ' '}</p>
		<h1>Il tuo garage</h1>
	</div>
</div>

{#if error}
	<div class="alert danger" role="alert"><Icon name="circle-alert" />{error}</div>
{:else if !data}
	<div class="stats">
		{#each [1, 2, 3, 4] as i (i)}<div class="skeleton" style="height:112px"></div>{/each}
	</div>
	<div class="skeleton section" style="height:320px"></div>
{:else if data.vehicles.length === 0}
	<div class="card empty">
		<span class="tile-ico"><Icon name="car" size={28} /></span>
		<h2>Benvenuto in MyGarage</h2>
		<p>Aggiungi la tua prima auto: bastano il nome e i km attuali. Le scadenze principali te le propongo io.</p>
		<a class="btn btn-primary" href="/auto/nuova"><Icon name="plus" />Aggiungi auto</a>
	</div>
{:else}
	<div class="stats">
		<a class="card stat" href="/scadenze?stato=overdue">
			<span class="stat-ico overdue"><Icon name="alert" /></span>
			<div><div class="stat-val num">{data.counts.overdue}</div><div class="stat-lbl">Scadute</div></div>
		</a>
		<a class="card stat" href="/scadenze?stato=soon">
			<span class="stat-ico soon"><Icon name="clock" /></span>
			<div><div class="stat-val num">{data.counts.soon}</div><div class="stat-lbl">In arrivo</div></div>
		</a>
		<a class="card stat" href="/scadenze?stato=ok">
			<span class="stat-ico ok"><Icon name="ok" /></span>
			<div><div class="stat-val num">{data.counts.ok}</div><div class="stat-lbl">In regola</div></div>
		</a>
		<a class="card stat" href="/spese">
			<span class="stat-ico primary"><Icon name="wallet" /></span>
			<div><div class="stat-val num">{fmtEuroRound(data.year_total_cents)}</div><div class="stat-lbl">Spese {data.year}</div></div>
		</a>
	</div>

	{#if data.counts.setup}
		<a class="alert info section" href="/scadenze?stato=setup">
			<Icon name="help" />
			<span>{data.counts.setup} scadenz{data.counts.setup === 1 ? 'a' : 'e'} da completare: indica quando sono state fatte l’ultima volta.</span>
		</a>
	{/if}

	<div class="grid-2 section">
		<section aria-labelledby="h-next">
			<div class="section-head">
				<h2 id="h-next">Prossime scadenze</h2>
				<a class="link" href="/scadenze">Vedi tutte <Icon name="chevron-right" size={16} /></a>
			</div>
			<div class="card">
				{#if data.upcoming.filter((s) => s.status !== 'setup').length}
					<ul class="list due-list">
						{#each data.upcoming.filter((s) => s.status !== 'setup') as s (s.id)}<DueRow {s} />{/each}
					</ul>
				{:else}
					<div class="empty">
						<span class="tile-ico"><Icon name="calendar" size={26} /></span>
						<p>Nessuna scadenza impostata.</p>
					</div>
				{/if}
			</div>
		</section>

		<section aria-labelledby="h-spend">
			<div class="section-head">
				<h2 id="h-spend">Spese {data.year}</h2>
				<a class="link" href="/spese">Dettaglio <Icon name="chevron-right" size={16} /></a>
			</div>
			<div class="card spend">
				<div class="spend-total"><span class="stat-lbl">Totale anno</span><span class="v num">{fmtEuroRound(data.year_total_cents)}</span></div>
				{#each data.year_by_category as c (c.category)}
					<div class="spend-row">
						<span class="lbl"><Icon name={categoryIcon[c.category] ?? 'wrench'} size={16} />{categoryLabel[c.category] ?? c.category}</span>
						<span class="amt num">{fmtEuroRound(c.total_cents)}</span>
						<div class="bar"><span style="width:{(c.total_cents / maxCat) * 100}%"></span></div>
					</div>
				{:else}
					<p class="subtle" style="margin:0">Nessuna spesa registrata quest’anno. Aggiungi il costo quando registri un intervento.</p>
				{/each}
			</div>
		</section>
	</div>

	<section class="section" aria-labelledby="h-cars">
		<div class="section-head">
			<h2 id="h-cars">Le tue auto</h2>
			<a class="link" href="/auto/nuova"><Icon name="plus" size={16} />Aggiungi</a>
		</div>
		<div class="cars">
			{#each data.vehicles as v (v.id)}<VehicleCard {v} />{/each}
		</div>
	</section>
{/if}

<style>
	.stats {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
	}
	@media (min-width: 768px) {
		.stats {
			grid-template-columns: repeat(4, minmax(0, 1fr));
		}
	}
	.stat {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		transition: transform 0.15s var(--ease);
	}
	.stat:hover {
		transform: translateY(-1px);
	}
	.stat-ico {
		width: 40px;
		height: 40px;
		border-radius: 12px;
		display: grid;
		place-items: center;
	}
	.stat-ico.overdue {
		background: var(--danger-soft);
		color: var(--danger);
	}
	.stat-ico.soon {
		background: var(--warn-soft);
		color: var(--warn);
	}
	.stat-ico.ok {
		background: var(--ok-soft);
		color: var(--ok);
	}
	.stat-ico.primary {
		background: var(--primary-soft);
		color: var(--primary);
	}
	.stat-val {
		font-size: 30px;
		font-weight: 800;
		letter-spacing: -0.02em;
		line-height: 1;
	}
	.stat-lbl {
		font-size: 14px;
		color: var(--fg-muted);
		font-weight: 500;
	}
	.due-list :global(li) {
		padding: 0;
		display: block;
	}
	.spend {
		padding: 18px;
		display: grid;
		gap: 14px;
	}
	.spend-total {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
	}
	.spend-total .v {
		font-size: 28px;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.spend-row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 6px 12px;
		font-size: 14px;
	}
	.spend-row .lbl {
		font-weight: 600;
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.spend-row .amt {
		font-weight: 700;
	}
	.spend-row .bar {
		grid-column: 1 / -1;
		height: 6px;
	}
</style>
