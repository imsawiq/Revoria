<template>
	<Teleport to="body" :disabled="!isFullscreen">
		<div
			class="console-layout flex min-h-0 flex-1 flex-col gap-4"
			:class="
				isFullscreen
					? `fixed inset-0 z-[120] h-screen w-screen overflow-auto bg-surface-1 p-6 py-8 ${isApp ? 'pt-12' : ''}`
					: ''
			"
		>
			<CollapsibleAdmonition
				v-if="ctx.crashAnalysis?.value && !isFullscreen"
				type="critical"
				:header="crashHeader"
				:items="crashItems"
				dismissible
				@dismiss="ctx.onDismissCrash?.()"
			/>

			<div class="flex items-center gap-2">
				<StyledInput
					v-model="searchQuery"
					:icon="SearchIcon"
					:placeholder="formatMessage(messages.searchLogs)"
					wrapper-class="flex-1"
					input-class="!h-10 !pl-10"
					clearable
				/>
				<div v-if="ctx.logSources?.value && ctx.activeLogSourceIndex" class="w-[220px]">
					<Combobox
						:model-value="ctx.activeLogSourceIndex.value"
						:options="logSourceOptions"
						@update:model-value="(value) => (ctx.activeLogSourceIndex!.value = value)"
					/>
				</div>
			</div>

			<div class="flex items-center justify-between gap-3">
				<ConsoleFilterPills
					v-model="activeFilters"
					:present-levels="presentLevels"
					@toggle="handleFilterToggle"
				/>
				<ConsoleActionButtons
					:show-clear="isLiveSource"
					:has-logs="hasLogs"
					:share-disabled="resolvedShareDisabled"
					:share-disabled-tooltip="resolvedShareDisabled ? formatMessage(messages.shareDisabled) : undefined"
					:sharing="isSharing"
					:fullscreen="isFullscreen"
					:show-delete="showDelete"
					:delete-disabled="resolvedDeleteDisabled"
					:delete-disabled-tooltip="ctx.deleteDisabledTooltip"
					@clear="handleClear"
					@share="handleShare"
					@toggle-fullscreen="toggleFullscreen"
					@delete="handleDelete"
				/>
			</div>

			<BaseTerminal
				ref="terminalRef"
				class="min-h-0 flex-1"
				:show-input="resolvedShowInput"
				:disable-input="resolvedDisableInput"
				:fullscreen="isFullscreen"
				:empty-state-type="ctx.emptyStateType"
				@command="handleCommand"
				@ready="handleTerminalReady"
			/>
		</div>
	</Teleport>
	<ShareModal
		ref="shareModal"
		:header="formatMessage(messages.shareHeader)"
		link
		:social-buttons="false"
	/>
	<NewModal
		ref="deleteModal"
		:header="formatMessage(messages.deleteLogFile)"
		:fade="'danger'"
		max-width="500px"
	>
		<div class="flex flex-col gap-6">
			<Admonition type="critical" :header="formatMessage(messages.irreversible)">
				{{ formatMessage(messages.deleteConfirmation) }}
			</Admonition>
		</div>
		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-4" @click="deleteModal?.hide()">
						<XIcon />
						{{ formatMessage(messages.cancel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button :disabled="isDeleting" @click="confirmDelete">
						<TrashIcon />
						{{ formatMessage(messages.delete) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { SearchIcon, TrashIcon, XIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { Teleport, computed, isRef, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import BaseTerminal from '#ui/components/base/BaseTerminal.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import type { CollapsibleAdmonitionItem } from '#ui/components/base/CollapsibleAdmonition.vue'
import CollapsibleAdmonition from '#ui/components/base/CollapsibleAdmonition.vue'
import Combobox from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import ShareModal from '#ui/components/modal/ShareModal.vue'
import { injectModrinthClient } from '#ui/providers'
import { injectModalBehavior } from '#ui/providers/modal-behavior'
import { injectNotificationManager } from '#ui/providers/web-notifications.ts'

import ConsoleActionButtons from './components/ConsoleActionButtons.vue'
import ConsoleFilterPills from './components/ConsoleFilterPills.vue'
import { colorize, rewriteTerminal, useConsoleFilters } from './composables'
import type { ConditionalLevel } from './composables/console-filtering'
import { injectConsoleManager } from './providers'
import type { LogLevel, LogLine } from './types'

const ctx = injectConsoleManager()
const client = injectModrinthClient()
const modalBehavior = injectModalBehavior(null)
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	searchLogs: {
		id: 'instance.logs.filter.placeholder',
		defaultMessage: 'Type to filter logs...',
	},
	shareHeader: {
		id: 'instance.logs.share.header',
		defaultMessage: 'Share Log',
	},
	shareDisabled: {
		id: 'instance.logs.share.disabled',
		defaultMessage: 'Sharing is unavailable offline',
	},
	deleteLogFile: {
		id: 'console.delete.header',
		defaultMessage: 'Delete log file',
	},
	irreversible: {
		id: 'console.delete.irreversible',
		defaultMessage: 'This is irreversible',
	},
	deleteConfirmation: {
		id: 'console.delete.confirmation',
		defaultMessage: 'Deleting this log file cannot be undone. Are you sure you want to continue?',
	},
	cancel: {
		id: 'button.cancel',
		defaultMessage: 'Cancel',
	},
	delete: {
		id: 'instance.logs.delete',
		defaultMessage: 'Delete',
	},
	failedDeleteTitle: {
		id: 'console.delete.failed-title',
		defaultMessage: 'Failed to delete log file',
	},
	failedShareTitle: {
		id: 'console.share.failed-title',
		defaultMessage: 'Failed to share logs',
	},
	unknownError: {
		id: 'error.unknown',
		defaultMessage: 'Unknown error.',
	},
})

const crashHeader = computed(() => {
	const problems = ctx.crashAnalysis?.value?.analysis.problems ?? []
	const count = problems.length
	return `${count} ${count === 1 ? 'problem' : 'problems'} detected`
})

const crashItems = computed<CollapsibleAdmonitionItem[]>(() => {
	const problems = ctx.crashAnalysis?.value?.analysis.problems ?? []
	return problems.map((problem) => ({
		title: problem.message,
		descriptions: problem.solutions.map((solution) => solution.message),
	}))
})

const terminalRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
const shareModal = ref<InstanceType<typeof ShareModal> | null>(null)
const deleteModal = ref<InstanceType<typeof NewModal> | null>(null)
const isDeleting = ref(false)
const searchQuery = ref('')
const isFullscreen = ref(false)
const isApp =
	typeof window !== 'undefined' && !!(window as Record<string, unknown>).__TAURI_INTERNALS__
const isSharing = ref(false)
const { activeFilters, toggleFilter, buildFilterPredicate } = useConsoleFilters()
const hasLogs = computed(() => ctx.logLines.value.length > 0)
const presentLevels = computed(() => {
	const levels = new Set<ConditionalLevel>()
	for (const line of ctx.logLines.value) {
		if (line.level === 'debug') levels.add('debug')
		if (line.level === 'trace') levels.add('trace')
		if (levels.size === 2) break
	}
	return levels
})
const isLiveSource = computed(() => {
	const sources = ctx.logSources?.value
	const index = ctx.activeLogSourceIndex?.value
	if (!sources || index === undefined) return true
	return sources[index]?.live ?? true
})
const logSourceOptions = computed(() =>
	(ctx.logSources?.value ?? []).map((source, index) => ({ value: index, label: source.name })),
)

function buildCombinedPredicate(): ((line: LogLine) => boolean) | null {
	const levelPredicate = buildFilterPredicate()
	const query = searchQuery.value.trim().toLowerCase()
	if (!levelPredicate && !query) return null
	return (line: LogLine) => {
		if (levelPredicate && !levelPredicate(line)) return false
		if (query && !line.text.toLowerCase().includes(query)) return false
		return true
	}
}

onBeforeUnmount(() => {
	if (isFullscreen.value) {
		document.body.style.overflow = ''
		modalBehavior?.onHide?.()
	}
})

let lastWrittenIndex = 0
let searchDebounce: ReturnType<typeof setTimeout> | null = null

const resolvedShowInput = computed(() => {
	const value = ctx.showCommandInput
	if (value === undefined) return false
	if (typeof value === 'boolean') return value
	return isRef(value) ? value.value : value
})

const resolvedDisableInput = computed(() => {
	const value = ctx.disableCommandInput
	if (!value) return false
	return isRef(value) ? value.value : value
})

const resolvedShareDisabled = computed(() => {
	const value = ctx.shareDisabled
	if (!value) return false
	return isRef(value) ? value.value : value
})

const showDelete = computed(() => !isLiveSource.value && ctx.onDelete != null)

const resolvedDeleteDisabled = computed(() => {
	const value = ctx.deleteDisabled
	if (!value) return false
	return isRef(value) ? value.value : value
})

function handleTerminalReady(_terminal: Terminal) {
	rewriteFiltered()
}

function handleFilterToggle(value: LogLevel | 'all') {
	toggleFilter(value)
	rewriteFiltered()
}

function activeSearchQuery(): string {
	return searchQuery.value.trim().toLowerCase()
}

function writeEmptyState() {
	terminalRef.value?.writeEmptyState()
	lastWrittenIndex = 0
}

function rewriteFiltered() {
	const term = terminalRef.value?.terminal
	if (!term) return
	const lines = ctx.logLines.value
	if (lines.length === 0 && isLiveSource.value) {
		writeEmptyState()
		return
	}
	terminalRef.value?.clearEmptyState()
	const predicate = buildCombinedPredicate()
	rewriteTerminal(term, lines, predicate, activeSearchQuery())
	lastWrittenIndex = lines.length
}

function toggleFullscreen() {
	isFullscreen.value = !isFullscreen.value
	if (isFullscreen.value) {
		document.body.style.overflow = 'hidden'
		modalBehavior?.onShow?.()
	} else {
		document.body.style.overflow = ''
		modalBehavior?.onHide?.()
	}
	nextTick(() => {
		terminalRef.value?.fit()
	})
}

watch(ctx.logLines, (lines, oldLines) => {
	const term = terminalRef.value?.terminal
	if (!term) return

	if (lines.length === 0 && isLiveSource.value) {
		writeEmptyState()
		return
	}

	if (
		terminalRef.value?.showingEmptyState ||
		lines !== oldLines ||
		lines.length < lastWrittenIndex
	) {
		terminalRef.value?.clearEmptyState()
		rewriteFiltered()
		return
	}

	const predicate = buildCombinedPredicate()
	const query = activeSearchQuery()
	const newLines: string[] = []
	for (let index = lastWrittenIndex; index < lines.length; index++) {
		if (!predicate || predicate(lines[index])) {
			newLines.push(colorize(lines[index], query))
		}
	}
	if (newLines.length > 0) {
		const buffer = term.buffer.active
		const onFreshLine = buffer.cursorX === 0
		const data = onFreshLine ? newLines.join('\r\n') : '\r\n' + newLines.join('\r\n')
		term.write(data)
	}
	lastWrittenIndex = lines.length
})

watch(searchQuery, () => {
	if (searchDebounce) clearTimeout(searchDebounce)
	searchDebounce = setTimeout(() => {
		rewriteFiltered()
	}, 200)
})

function handleCommand(cmd: string) {
	ctx.sendCommand?.(cmd)
}

function handleClear() {
	terminalRef.value?.reset()
	lastWrittenIndex = 0
	ctx.onClear?.()
}

function handleDelete() {
	deleteModal.value?.show()
}

async function confirmDelete() {
	if (!ctx.onDelete) return
	isDeleting.value = true
	try {
		await ctx.onDelete()
		deleteModal.value?.hide()
	} catch (error) {
		console.error('Failed to delete log file:', error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedDeleteTitle),
			text: typeof error === 'string' ? error : formatMessage(messages.unknownError),
		})
	} finally {
		isDeleting.value = false
	}
}

async function handleShare() {
	const predicate = buildCombinedPredicate()
	const lines = predicate ? ctx.logLines.value.filter(predicate) : ctx.logLines.value
	const content = lines.map((line) => line.text).join('\n')

	isSharing.value = true
	try {
		const data = await client.mclogs.logs_v1.create(content)
		if (data.url) {
			shareModal.value?.show(data.url)
		}
	} catch (error) {
		console.error('Failed to share logs:', error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedShareTitle),
			text: typeof error === 'string' ? error : formatMessage(messages.unknownError),
		})
	} finally {
		isSharing.value = false
	}
}
</script>
