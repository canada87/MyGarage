<script lang="ts">
	import Icon from './Icon.svelte';
	import { documentKinds, fmtBytes, fmtDate } from '#lib/format.ts';
	import { openDialog } from '#lib/state.svelte.ts';
	import type { DocumentFile } from '#lib/types.ts';

	let { docs, byYear = false }: { docs: DocumentFile[]; byYear?: boolean } = $props();

	const dateOf = (d: DocumentFile) => d.doc_date ?? d.uploaded_at.slice(0, 10);

	/** Newest year first; the server already sorts documents by date. */
	const groups = $derived.by(() => {
		if (!byYear) return [{ year: '', docs }];
		const map = new Map<string, DocumentFile[]>();
		for (const d of docs) {
			const y = dateOf(d).slice(0, 4);
			if (!map.has(y)) map.set(y, []);
			map.get(y)!.push(d);
		}
		return [...map.entries()].sort(([a], [b]) => b.localeCompare(a)).map(([year, docs]) => ({ year, docs }));
	});
</script>

{#if docs.length}
	{#each groups as g (g.year)}
		{#if g.year}
			<h3 class="year">{g.year} <span class="count">· {g.docs.length} document{g.docs.length === 1 ? 'o' : 'i'}</span></h3>
		{/if}
		<div class="card">
			<ul class="list">
				{#each g.docs as d (d.id)}
					<li>
						<span class="tile-ico"><Icon name={d.mime.startsWith('image/') ? 'image' : 'file'} size={18} /></span>
						<a class="grow info" href="/api/documents/{d.id}/file" target="_blank" rel="noopener" data-sveltekit-reload>
							<span class="t">{d.title}</span>
							<span class="s">{documentKinds[d.kind] ?? d.kind} · {fmtDate(dateOf(d))} · {fmtBytes(d.size)}{d.event_id ? ' · da un intervento' : ''}</span>
						</a>
						<a class="icon-btn ghost" href="/api/documents/{d.id}/file?download=true" download data-sveltekit-reload aria-label="Scarica {d.title}">
							<Icon name="download" size={18} />
						</a>
						<button class="icon-btn ghost" type="button" aria-label="Modifica {d.title}" onclick={() => openDialog({ kind: 'document', doc: d })}>
							<Icon name="pencil" size={18} />
						</button>
					</li>
				{/each}
			</ul>
		</div>
	{/each}
{:else}
	<div class="card">
		<div class="empty">
			<p>Nessun documento. Carica libretto, polizze e fatture: li ritrovi qui anche dal telefono.</p>
		</div>
	</div>
{/if}

<style>
	.info {
		display: grid;
		min-height: 44px;
		align-content: center;
	}
	.year {
		margin: 16px 0 8px;
		color: var(--fg-subtle);
		font-size: 14px;
	}
	.year:first-child {
		margin-top: 0;
	}
	.count {
		font-weight: 500;
	}
</style>
