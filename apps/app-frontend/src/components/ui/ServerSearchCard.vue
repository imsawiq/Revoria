<template>
	<ProjectCard
		type="server"
		:title="project.name"
		:link="`/project/${project.slug ?? project.project_id}`"
		:icon-url="project.icon_url ?? undefined"
		:summary="project.summary ?? ''"
		:tags="displayCategories"
		:all-tags="project.categories ?? []"
		:downloads="playersOnline"
		:followers="recentPlays"
		:date-updated="updatedAt"
		:date-published="createdAt"
		layout="list"
		@click="emit('open')"
		@contextmenu.prevent.stop="(event) => emit('contextmenu', event)"
	>
		<template #tags-prefix>
			<span class="server-metric" :title="statusText">
				<span class="presence-dot" :class="statusOnline ? 'is-online' : 'is-offline'" />
				{{ formatNumber(playersOnline, false) }}
			</span>
			<span v-if="regionLabel" class="server-region-tag" :title="regionLabel">
				{{ regionLabel }}
			</span>
			<span class="server-ping-tag" :class="`is-${pingState}`" :title="pingTitle">
				<SignalIcon />
				{{ pingText }}
			</span>
		</template>
		<template #tags-suffix>
			<span
				v-if="overflowCategoryTags.length > 0"
				v-tooltip="overflowCategoryTooltip"
				class="server-region-tag smart-clickable:allow-pointer-events"
			>
				+{{ overflowCategoryTags.length }}
			</span>
			<span
				v-if="serverModpackContent"
				v-tooltip="
					serverModpackContent.showCustomModpackTooltip
						? 'This project uses a custom modpack'
						: serverModpackContent.name
				"
				class="server-modpack-content smart-clickable:allow-pointer-events"
			>
				<img
					v-if="serverModpackContent.icon"
					:src="serverModpackContent.icon"
					alt=""
					loading="lazy"
				/>
				<span>{{ serverModpackContent.name }}</span>
			</span>
		</template>
		<div class="server-actions">
			<ButtonStyled circular>
				<button
					v-tooltip="'Add server to instance'"
					:disabled="adding"
					@click.stop="emit('add')"
				>
					<PlusIcon />
				</button>
			</ButtonStyled>
			<ButtonStyled v-if="running" color="red" type="outlined">
				<button class="shrink-0 no-wrap" @click.stop="emit('stop')">
					<StopCircleIcon />
					{{ formatMessage(messages.actionStop) }}
				</button>
			</ButtonStyled>
			<ButtonStyled v-else color="brand" type="outlined">
				<button :disabled="installing" class="shrink-0 no-wrap" @click.stop="emit('play')">
					<PlayIcon />
					{{
						installing
							? formatMessage(messages.actionInstalling)
							: formatMessage(messages.actionPlay)
					}}
				</button>
			</ButtonStyled>
		</div>
	</ProjectCard>
</template>

<script setup>
import {
	PlayIcon,
	PlusIcon,
	SignalIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import { formatCategory, formatNumber } from '@modrinth/utils'
import { ButtonStyled, ProjectCard } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	ping: {
		type: Number,
		default: undefined,
	},
	running: {
		type: Boolean,
		default: false,
	},
	installing: {
		type: Boolean,
		default: false,
	},
	canAdd: {
		type: Boolean,
		default: false,
	},
	adding: {
		type: Boolean,
		default: false,
	},
	tagDefinitions: {
		type: Object,
		default: () => ({}),
	},
})

const emit = defineEmits(['open', 'add', 'play', 'stop', 'contextmenu'])
const { formatMessage } = useVIntl()

const messages = defineMessages({
	actionStop: {
		id: 'browse.server.action.stop',
		defaultMessage: 'Stop',
	},
	actionPlay: {
		id: 'browse.server.action.play',
		defaultMessage: 'Play',
	},
	actionInstalling: {
		id: 'browse.server.action.installing',
		defaultMessage: 'Installing...',
	},
	statusOnline: {
		id: 'browse.server.status.online',
		defaultMessage: 'Server online',
	},
	statusOffline: {
		id: 'browse.server.status.offline',
		defaultMessage: 'Server offline',
	},
	playersOnline: {
		id: 'browse.server.players-online',
		defaultMessage: '{count} players online',
	},
	recentPlays: {
		id: 'browse.server.recent-plays',
		defaultMessage: '{count} recent plays',
	},
	pingUnavailable: {
		id: 'browse.server.ping.unavailable',
		defaultMessage: 'Ping unavailable',
	},
	pingValue: {
		id: 'browse.server.ping.value',
		defaultMessage: '{ping} ms',
	},
})
const FALLBACK_TAG_ICON =
	'<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M8 2v12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>'

const resolveTagIcon = (icon) => {
	if (typeof icon !== 'string') return FALLBACK_TAG_ICON
	const trimmed = icon.trim()
	if (!trimmed) return FALLBACK_TAG_ICON
	if (
		trimmed.startsWith('<') ||
		trimmed.startsWith('http://') ||
		trimmed.startsWith('https://') ||
		trimmed.startsWith('data:image/') ||
		trimmed.startsWith('/')
	) {
		return trimmed
	}
	return FALLBACK_TAG_ICON
}

const toFiniteNumber = (value) => {
	const numeric = Number(value)
	return Number.isFinite(numeric) ? numeric : 0
}
const toOptionalPing = (value) => {
	const numeric = Number(value)
	return Number.isFinite(numeric) && numeric >= 0 ? numeric : undefined
}

const playersOnline = computed(() => toFiniteNumber(props.project.minecraft_java_server?.ping?.data?.players_online))
const recentPlays = computed(() => toFiniteNumber(props.project.minecraft_java_server?.verified_plays_2w))
const statusOnline = computed(() => Boolean(props.project.minecraft_java_server?.ping?.data))
const projectPing = computed(() => {
	const pingData = props.project?.minecraft_java_server?.ping
	return (
		toOptionalPing(props.ping) ??
		toOptionalPing(pingData?.latency) ??
		toOptionalPing(pingData?.latency_ms) ??
		toOptionalPing(pingData?.avg) ??
		toOptionalPing(props.project?.minecraft_java_server?.ping_ms)
	)
})
const statusText = computed(() =>
	statusOnline.value ? formatMessage(messages.statusOnline) : formatMessage(messages.statusOffline),
)
const pingState = computed(() => {
	if (!statusOnline.value) return 'offline'
	if (!Number.isFinite(projectPing.value)) return 'good'
	if (projectPing.value < 100) return 'good'
	if (projectPing.value < 250) return 'warn'
	return 'bad'
})
const pingText = computed(() =>
	Number.isFinite(projectPing.value)
		? formatMessage(messages.pingValue, { ping: Math.round(projectPing.value) })
		: formatMessage(messages.pingUnavailable),
)
const pingTitle = computed(() => pingText.value)
const playersOnlineText = computed(() =>
	formatMessage(messages.playersOnline, { count: formatNumber(playersOnline.value, false) }),
)
const createdAt = computed(
	() => props.project.date_created ?? props.project.date_modified ?? '1970-01-01T00:00:00.000Z',
)
const updatedAt = computed(() => props.project.date_modified ?? createdAt.value)
const serverModpackContent = computed(() => {
	const content = props.project?.minecraft_java_server?.content
	if (content?.kind !== 'modpack' || !content.project_name) return null
	return {
		name: content.project_name,
		icon: content.project_icon ?? '',
		showCustomModpackTooltip: content.project_id === props.project.project_id,
	}
})
const regionLabel = computed(() => {
	const region = String(props.project?.minecraft_server?.region ?? '')
	if (!region) return ''
	return region
		.split('_')
		.map((part) => (part ? `${part[0].toUpperCase()}${part.slice(1)}` : ''))
		.join(' ')
})

const loaderIndex = computed(() => {
	const map = new Map()
	for (const loader of props.tagDefinitions?.loaders ?? []) {
		if (loader?.name) map.set(String(loader.name).toLowerCase(), loader)
	}
	return map
})
const categoryIndex = computed(() => {
	const map = new Map()
	for (const category of props.tagDefinitions?.categories ?? []) {
		if (category?.name) map.set(String(category.name).toLowerCase(), category)
	}
	return map
})

const allCategoryTags = computed(() =>
	(props.project.display_categories ?? props.project.categories ?? []).reduce((acc, rawTag) => {
		const tag = String(rawTag ?? '').trim()
		if (!tag) return acc
		const key = tag.toLowerCase()
		if (acc.seen.has(key)) return acc
		acc.seen.add(key)
		acc.list.push(tag)
		return acc
	}, { list: [], seen: new Set() }).list,
)
const visibleCategoryTags = computed(() => allCategoryTags.value.slice(0, 2))
const overflowCategoryTags = computed(() => allCategoryTags.value.slice(2))
const overflowCategoryTooltip = computed(() =>
	overflowCategoryTags.value.map((tag) => formatCategory(tag)).join(', '),
)

const displayCategories = computed(() =>
	visibleCategoryTags.value
		.map((tag) => {
			const name = String(tag ?? '').trim()
			if (!name) return null
			const key = name.toLowerCase()
			const loader = loaderIndex.value.get(key)
			const category = categoryIndex.value.get(key)
			const matched = loader ?? category
			return {
				name,
				icon: resolveTagIcon(matched?.icon),
				isLoader: Boolean(loader),
			}
		})
		.filter(Boolean),
)
</script>

<style scoped>
.server-actions {
	display: flex;
	gap: 0.5rem;
	align-items: center;
}

.server-metric {
	display: inline-flex;
	align-items: center;
	gap: 0.28rem;
	color: var(--color-text);
	font-size: 0.95rem;
	padding: 0 0.15rem;
	border: none;
	background: transparent;
	line-height: 1.1;
	min-height: 1.2rem;
}

.server-region-tag {
	display: inline-flex;
	align-items: center;
	gap: 0.35rem;
	color: var(--color-secondary);
	font-size: 0.95rem;
	padding: 0.12rem 0.62rem;
	border: 1px solid var(--color-button-border);
	border-radius: 999px;
	background: color-mix(in srgb, var(--color-button-bg) 92%, transparent 8%);
	line-height: 1.1;
}

.server-ping-tag {
	display: inline-flex;
	align-items: center;
	gap: 0.35rem;
	font-size: 0.95rem;
	padding: 0.12rem 0.62rem;
	border: 1px solid var(--color-button-border);
	border-radius: 999px;
	line-height: 1.1;
}

.server-ping-tag.is-good,
.server-ping-tag.is-unknown {
	border-color: var(--color-button-border);
	color: var(--color-secondary);
	background: color-mix(in srgb, var(--color-button-bg) 92%, transparent 8%);
}

.server-ping-tag.is-good {
	border-color: var(--color-brand);
	color: var(--color-brand);
	background: var(--color-green-highlight);
}

.server-ping-tag.is-warn {
	border-color: var(--color-orange);
	color: var(--color-orange);
	background: var(--color-orange-highlight);
}

.server-ping-tag.is-bad,
.server-ping-tag.is-offline {
	border-color: var(--color-red);
	color: var(--color-red);
	background: var(--color-red-highlight);
}

.server-modpack-content {
	display: inline-flex;
	align-items: center;
	gap: 0.38rem;
	max-width: 17rem;
	color: var(--color-secondary);
	font-size: 0.92rem;
}

.server-modpack-content img {
	width: 1rem;
	height: 1rem;
	border-radius: 0.2rem;
	object-fit: cover;
	flex-shrink: 0;
}

.server-modpack-content span {
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.presence-dot {
	display: inline-flex;
	width: 0.72rem;
	height: 0.72rem;
	border-radius: 999px;
	border: none;
	background: currentcolor;
	box-sizing: border-box;
}

.presence-dot.is-online {
	color: var(--color-brand);
}

.presence-dot.is-offline {
	color: var(--color-red);
}

.server-metric :deep(svg),
.server-region-tag :deep(svg),
.server-ping-tag :deep(svg) {
	width: 0.9rem;
	height: 0.9rem;
}
</style>
