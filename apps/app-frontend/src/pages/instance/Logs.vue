<template>
	<div class="flex h-full flex-col gap-4">
		<div class="revoria-console-theme">
			<ConsolePageLayout />
		</div>

		<Card class="crash-card">
			<div class="crash-header">
				<div>
					<h2 class="m-0 text-lg font-bold">{{ crashText.header }}</h2>
					<p class="m-0 text-secondary">{{ crashText.description }}</p>
				</div>
				<Button :disabled="crashLoading" @click="loadCrashLogs">{{ crashText.refresh }}</Button>
			</div>
			<div class="crash-grid">
				<div class="crash-settings">
					<label class="label">{{ formatMessage(messages.endpointLabel) }}</label>
					<input v-model="crashSettings.endpoint" class="crash-input" />
					<label class="label">{{ crashText.model }}</label>
					<input v-model="crashSettings.model" class="crash-input" />
					<label class="label">{{ crashText.apiKey }}</label>
					<input v-model="crashSettings.apiKey" class="crash-input" type="password" />
					<div class="hint">
						<p class="m-0"><strong>{{ crashText.apiTutorialTitle }}</strong></p>
						<ol class="m-0 pl-5">
							<li>{{ crashText.apiTutorialStep1 }}</li>
							<li>{{ crashText.apiTutorialStep2 }}</li>
							<li>{{ crashText.apiTutorialStep3 }}</li>
						</ol>
					</div>
				</div>
				<div class="crash-logs">
					<DropdownSelect
						v-model="selectedCrashIndex"
						:default-value="0"
						:name="formatMessage(messages.crashLogLabel)"
						class="crash-select"
						:options="crashLogs.map((_, index) => index)"
						:display-name="(option) => crashLogs[option]?.name"
						:disabled="crashLogs.length === 0"
					/>
					<div v-if="!crashLogs.length" class="empty-state">{{ crashText.empty }}</div>
					<div v-else class="crash-meta">
						<span class="text-xs text-secondary">
							{{ crashText.size }}: {{ formatBytes(crashLogs[selectedCrashIndex]?.size ?? 0) }}
						</span>
					</div>
					<div class="crash-actions">
						<Button :disabled="!props.instance?.path" @click="openCrashFolder">
							<FolderOpenIcon aria-hidden="true" />
							{{ crashText.folder }}
						</Button>
						<Button
							color="primary"
							:disabled="crashLoading || !crashLogs[selectedCrashIndex]"
							@click="analyzeCrashLog"
						>
							{{ crashLoading ? crashText.analyzing : crashText.analyze }}
						</Button>
					</div>
				</div>
			</div>
		</Card>
	</div>

	<ModalWrapper ref="crashModal" :header="crashText.result">
		<div class="modal-content">
			<div class="analysis-text markdown-body" v-html="crashResultHtml" />
		</div>
	</ModalWrapper>
</template>

<script setup>
import { FolderOpenIcon } from '@modrinth/assets'
import {
	Button,
	Card,
	ConsolePageLayout,
	DropdownSelect,
	injectNotificationManager,
	provideConsoleManager,
} from '@modrinth/ui'
import { formatBytes, renderString } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import { computed, onMounted, onUnmounted, ref, shallowRef, triggerRef, watch, watchEffect } from 'vue'
import { useRoute } from 'vue-router'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { useInstanceConsole } from '@/composables/useInstanceConsole'
import { process_listener } from '@/helpers/events.js'
import {
	delete_logs_by_filename,
	get_game_log_cursor,
	get_latest_log_cursor,
	get_output_by_filename,
} from '@/helpers/logs.js'
import { get_by_profile_path } from '@/helpers/process.js'
import { get_full_path } from '@/helpers/profile'
import { openPath } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()
const route = useRoute()
const { formatMessage } = useVIntl()

const props = defineProps({
	instance: {
		type: Object,
		default() {
			return {}
		},
	},
	options: {
		type: Object,
		default() {
			return {}
		},
	},
	openSettings: {
		type: Function,
		default: null,
	},
	offline: {
		type: Boolean,
		default() {
			return false
		},
	},
	playing: {
		type: Boolean,
		default() {
			return false
		},
	},
	versions: {
		type: Array,
		default() {
			return []
		},
	},
	installed: {
		type: Boolean,
		default() {
			return false
		},
	},
})

defineEmits(['play', 'stop'])

const messages = defineMessages({
	endpointLabel: {
		id: 'instance.logs.crash.endpoint',
		defaultMessage: 'Endpoint',
	},
	crashLogLabel: {
		id: 'instance.logs.crash.dropdown',
		defaultMessage: 'Crash log',
	},
	crashHeader: {
		id: 'instance.logs.crash.header',
		defaultMessage: 'Crash Log Checker',
	},
	crashDescription: {
		id: 'instance.logs.crash.description',
		defaultMessage: 'Analyzes this instance’s crash logs via the Ollama cloud model.',
	},
	crashRefresh: {
		id: 'instance.logs.crash.refresh',
		defaultMessage: 'Refresh',
	},
	crashModel: {
		id: 'instance.logs.crash.model',
		defaultMessage: 'Model',
	},
	crashApiKey: {
		id: 'instance.logs.crash.api-key',
		defaultMessage: 'API key',
	},
	crashApiTutorialTitle: {
		id: 'instance.logs.crash.api-tutorial.title',
		defaultMessage: 'How to get an Ollama API key:',
	},
	crashApiTutorialStep1: {
		id: 'instance.logs.crash.api-tutorial.step1',
		defaultMessage: 'Sign in at https://ollama.com/',
	},
	crashApiTutorialStep2: {
		id: 'instance.logs.crash.api-tutorial.step2',
		defaultMessage: 'Open settings keys: https://ollama.com/settings/keys',
	},
	crashApiTutorialStep3: {
		id: 'instance.logs.crash.api-tutorial.step3',
		defaultMessage: 'Click Add API Key, copy it, and paste it into the API key field above.',
	},
	crashEmpty: {
		id: 'instance.logs.crash.empty',
		defaultMessage: 'No crash logs found.',
	},
	crashSize: {
		id: 'instance.logs.crash.size',
		defaultMessage: 'Size',
	},
	crashFolder: {
		id: 'instance.logs.crash.folder',
		defaultMessage: 'Crash logs folder',
	},
	crashAnalyzing: {
		id: 'instance.logs.crash.analyzing',
		defaultMessage: 'Analyzing...',
	},
	crashAnalyze: {
		id: 'instance.logs.crash.analyze',
		defaultMessage: 'Analyze crash log',
	},
	crashResult: {
		id: 'instance.logs.crash.result',
		defaultMessage: 'Analysis result',
	},
	crashFail: {
		id: 'instance.logs.crash.fail',
		defaultMessage: 'Failed to get a response from the model.',
	},
	deleteLatestTooltip: {
		id: 'instance.logs.delete-latest-running',
		defaultMessage: 'Cannot delete latest.log while the instance is running',
	},
	liveLog: {
		id: 'instance.logs.live',
		defaultMessage: 'Live',
	},
	unknownLog: {
		id: 'instance.logs.unknown',
		defaultMessage: 'Unknown',
	},
})

const profilePathId = computed(() =>
	typeof route.params.id === 'string' ? route.params.id : String(route.params.id ?? ''),
)
const {
	liveConsole,
	historicalConsole,
	hydrate,
	getHistoricalLogs,
	getHistoricalContent,
	setHistoricalContent,
	invalidate,
	clearLive,
} = useInstanceConsole(profilePathId.value)

await hydrate()

function buildLogList(rawLogs) {
	return [
		{ name: formatMessage(messages.liveLog), live: true },
		...rawLogs
			.filter(
				(log) =>
					log.filename !== 'latest_stdout.log' &&
					log.filename !== 'latest_stdout' &&
					log.filename !== 'launcher_log.txt' &&
					log.stdout !== '' &&
					(log.filename.includes('.log') || log.filename.endsWith('.txt')),
			)
			.map((log) => ({
				...log,
				name: log.filename || formatMessage(messages.unknownLog),
			})),
	]
}

const logs = ref(buildLogList([]))

if (props.instance?.path) {
	void getHistoricalLogs(props.instance.path)
		.then((allLogs) => {
			logs.value = buildLogList(allLogs)
		})
		.catch(handleError)
}

const selectedLogIndex = ref(0)
const isLive = computed(() => selectedLogIndex.value === 0)
const filteredLogs = computed(() =>
	props.playing ? logs.value.filter((log) => log.live || log.name !== 'latest.log') : logs.value,
)
const logSources = computed(() =>
	filteredLogs.value.map((log, index) => ({
		id: String(index),
		name: log?.name ?? `Log ${index}`,
		live: log?.live ?? false,
	})),
)
const activeConsole = computed(() => (isLive.value ? liveConsole : historicalConsole))
const logLines = shallowRef(activeConsole.value.output.value)

watchEffect(() => {
	logLines.value = activeConsole.value.output.value
	triggerRef(logLines)
})

const selectedLog = computed(() => filteredLogs.value[selectedLogIndex.value])
const deleteDisabled = computed(() => {
	const log = selectedLog.value
	if (!log || log.live) return true
	return log.filename === 'latest.log' && props.playing
})

const currentLiveLogCursor = ref(0)

async function pollLiveLog() {
	if (!props.instance?.path || !profilePathId.value) return
	const processes = await get_by_profile_path(profilePathId.value).catch(() => [])
	if (!processes?.length) return

	const hasRecoveredProcess = processes.some((process) => process.recovered)
	const cursorData = await (
		hasRecoveredProcess ? get_game_log_cursor : get_latest_log_cursor
	)(props.instance.path, currentLiveLogCursor.value).catch(handleError)

	if (!cursorData) return
	if (cursorData.new_file) {
		liveConsole.clear()
		currentLiveLogCursor.value = 0
	}
	if (cursorData.output) {
		liveConsole.addLegacyLog(cursorData.output)
	}
	currentLiveLogCursor.value = cursorData.cursor
}

async function deleteSelectedLog() {
	const log = selectedLog.value
	if (!log || log.live) return
	await delete_logs_by_filename(props.instance.path, log.log_type, log.filename).catch(handleError)
	invalidate()
	const freshLogs = await getHistoricalLogs(props.instance.path)
	logs.value = buildLogList(freshLogs)
	selectedLogIndex.value = 0
}

provideConsoleManager({
	logLines,
	logSources,
	activeLogSourceIndex: selectedLogIndex,
	showCommandInput: false,
	loading: ref(false),
	onClear: () => {
		if (!isLive.value) return
		currentLiveLogCursor.value = 0
		void clearLive()
	},
	onDelete: deleteSelectedLog,
	deleteDisabled,
	deleteDisabledTooltip: formatMessage(messages.deleteLatestTooltip),
	shareDisabled: computed(() => props.offline),
	emptyStateType: 'instance',
})

watch(selectedLogIndex, async (newIndex) => {
	if (newIndex === 0) return
	const log = filteredLogs.value[newIndex]
	if (!log) return

	const cached = getHistoricalContent(log.filename)
	if (cached) {
		historicalConsole.clear()
		historicalConsole.addLegacyLog(cached)
		return
	}

	const output = await get_output_by_filename(props.instance.path, log.log_type, log.filename).catch(
		handleError,
	)
	if (output) {
		setHistoricalContent(log.filename, output)
		historicalConsole.clear()
		historicalConsole.addLegacyLog(output)
	}
})

const crashModal = ref(null)
const crashResult = ref('')
const crashLoading = ref(false)
const crashLogs = ref([])
const selectedCrashIndex = ref(0)
const crashResultHtml = computed(() => renderString(crashResult.value || ''))
const crashSettings = useStorage('crash-log-settings', {
	endpoint: 'https://ollama.com/api/chat',
	model: 'glm-5:cloud',
	apiKey: '',
})
const launcherLanguage = useStorage('launcher-language', 'ru')
const isRussian = computed(() => launcherLanguage.value === 'ru')
const crashText = computed(() => ({
	header: formatMessage(messages.crashHeader),
	description: formatMessage(messages.crashDescription),
	refresh: formatMessage(messages.crashRefresh),
	model: formatMessage(messages.crashModel),
	apiKey: formatMessage(messages.crashApiKey),
	apiTutorialTitle: formatMessage(messages.crashApiTutorialTitle),
	apiTutorialStep1: formatMessage(messages.crashApiTutorialStep1),
	apiTutorialStep2: formatMessage(messages.crashApiTutorialStep2),
	apiTutorialStep3: formatMessage(messages.crashApiTutorialStep3),
	empty: formatMessage(messages.crashEmpty),
	size: formatMessage(messages.crashSize),
	folder: formatMessage(messages.crashFolder),
	analyzing: formatMessage(messages.crashAnalyzing),
	analyze: formatMessage(messages.crashAnalyze),
	result: formatMessage(messages.crashResult),
	fail: formatMessage(messages.crashFail),
}))

async function openCrashFolder() {
	if (!props.instance?.path) return
	const fullPath = await get_full_path(props.instance.path).catch(handleError)
	if (!fullPath) return
	await openPath(`${fullPath}/crash-reports`).catch(handleError)
}

async function loadCrashLogs() {
	if (!props.instance?.path) return
	const fullPath = await get_full_path(props.instance.path).catch(handleError)
	if (!fullPath) return
	const files = await invoke('plugin:utils|list_dir_files', {
		path: `${fullPath}/crash-reports`,
		extensions: ['.txt', '.log'],
	}).catch(handleError)
	crashLogs.value = (files ?? []).filter((file) => file.name?.startsWith('crash-'))
	selectedCrashIndex.value = 0
}

async function analyzeCrashLog() {
	const selected = crashLogs.value[selectedCrashIndex.value]
	if (!selected) return
	crashLoading.value = true
	crashResult.value = ''
	try {
		const content = await invoke('plugin:utils|read_text_file', {
			path: selected.path,
		}).catch(handleError)
		if (!content) return
		const prompt = isRussian.value
			? `Ты эксперт по Minecraft. Проанализируй этот crash-лог, объясни причину простыми словами и затем дай 3–5 конкретных шагов для исправления.\n\nCRASH LOG:\n${content}`
			: `You are a Minecraft expert. Analyze this crash log, explain the root cause in plain English, then list 3-5 concrete steps to fix it.\n\nCRASH LOG:\n${content}`
		const raw = await invoke('plugin:utils|ollama_chat', {
			endpoint: crashSettings.value.endpoint,
			model: crashSettings.value.model,
			apiKey: crashSettings.value.apiKey || null,
			prompt,
		}).catch(handleError)
		if (!raw) return
		let data = null
		try {
			data = JSON.parse(raw)
		} catch {
			data = null
		}
		crashResult.value =
			data?.message?.content ?? data?.response ?? data?.output ?? raw ?? crashText.value.fail
		crashModal.value?.show()
	} catch (error) {
		handleError(error)
	} finally {
		crashLoading.value = false
	}
}

const pollInterval = ref(null)
let unlistenProcesses = null

onMounted(async () => {
	await loadCrashLogs()
	await pollLiveLog()

	pollInterval.value = setInterval(() => {
		void pollLiveLog()
	}, 250)

	unlistenProcesses = await process_listener(async (event) => {
		if (event.profile_path_id && event.profile_path_id !== profilePathId.value) return

		if (event.event === 'launched') {
			liveConsole.clear()
			currentLiveLogCursor.value = 0
			invalidate()
			selectedLogIndex.value = 0
		}
		if (event.event === 'finished') {
			invalidate()
			currentLiveLogCursor.value = 0
			const freshLogs = await getHistoricalLogs(props.instance.path)
			logs.value = buildLogList(freshLogs)
			await loadCrashLogs()
		}
	}).catch(() => null)
})

onUnmounted(() => {
	if (pollInterval.value) clearInterval(pollInterval.value)
	unlistenProcesses?.()
})
</script>

<style scoped lang="scss">
.crash-card {
	padding: 1rem;
	border: 1px solid var(--glass-border);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 96%, transparent);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur));
}

.crash-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
	margin-bottom: 1rem;
}

.crash-grid {
	display: grid;
	grid-template-columns: minmax(18rem, 22rem) minmax(0, 1fr);
	gap: 1rem;
}

.crash-settings,
.crash-logs {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	padding: 1rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
	border-radius: 1rem;
	background: color-mix(in srgb, var(--color-raised-bg) 92%, var(--color-glass-bg) 8%);
}

.label {
	font-size: 0.8rem;
	font-weight: 700;
	color: var(--color-secondary);
}

.crash-input {
	width: 100%;
	border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
	border-radius: 0.9rem;
	background: color-mix(in srgb, var(--color-button-bg) 88%, var(--color-raised-bg) 12%);
	color: var(--color-contrast);
	padding: 0.8rem 0.9rem;
}

.crash-input:focus {
	outline: none;
	border-color: color-mix(in srgb, var(--color-brand) 42%, var(--glass-border));
	box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-brand-highlight) 68%, transparent);
}

.hint {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: 0.9rem 1rem;
	border-radius: 1rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
	background: color-mix(in srgb, var(--color-button-bg) 84%, var(--color-raised-bg) 16%);
	color: var(--color-secondary);
}

.empty-state {
	display: grid;
	place-items: center;
	min-height: 8rem;
	border: 1px dashed color-mix(in srgb, var(--glass-border) 82%, transparent);
	border-radius: 1rem;
	color: var(--color-secondary);
	background: color-mix(in srgb, var(--color-raised-bg) 94%, var(--color-button-bg) 6%);
}

.crash-actions {
	display: flex;
	flex-wrap: wrap;
	gap: 0.75rem;
	margin-top: auto;
}

.crash-card :deep(.btn-wrapper) {
	--_bg: color-mix(in srgb, var(--color-button-bg) 88%, var(--color-raised-bg) 12%);
	--_text: var(--color-base);
	--_hover-bg: color-mix(in srgb, var(--color-button-bg-hover) 84%, transparent);
	--_hover-text: var(--color-contrast);
}

.crash-card :deep(.btn-wrapper[color='primary']),
.crash-card :deep(.btn-wrapper[color='brand']) {
	--_bg: color-mix(in srgb, var(--color-button-bg-selected) 78%, transparent);
	--_text: var(--color-button-text-selected);
	--_hover-bg: color-mix(in srgb, var(--color-button-bg-selected) 92%, transparent);
	--_hover-text: var(--color-button-text-selected);
}

.modal-content {
	max-height: min(70vh, 44rem);
	overflow: auto;
	padding-right: 0.25rem;
}

.analysis-text {
	line-height: 1.6;
}

.revoria-console-theme {
	--surface-1: color-mix(in srgb, var(--color-glass-bg-strong) 96%, transparent);
	--surface-1_5: color-mix(in srgb, var(--color-glass-bg-strong) 88%, var(--color-raised-bg) 12%);
	--surface-2: color-mix(in srgb, var(--color-raised-bg) 96%, transparent);
	--surface-2_5: color-mix(in srgb, var(--color-raised-bg) 82%, var(--color-button-bg) 18%);
	--surface-3: color-mix(in srgb, var(--color-button-bg) 90%, var(--color-raised-bg) 10%);
	--surface-4: color-mix(in srgb, var(--glass-border) 84%, transparent);
	--surface-5: color-mix(in srgb, var(--glass-border) 94%, var(--color-raised-bg) 6%);
	--color-button-bg: color-mix(in srgb, var(--color-button-bg) 92%, var(--color-raised-bg) 8%);
	--color-button-bg-hover: color-mix(in srgb, var(--color-button-bg-hover) 92%, var(--color-raised-bg) 8%);
	--color-button-bg-active: color-mix(
		in srgb,
		var(--color-button-bg-selected) 62%,
		var(--color-button-bg-hover) 38%
	);
}

.revoria-console-theme :deep(.console-layout) {
	min-height: 100%;
}

.revoria-console-theme :deep(.console-layout .iconified-input),
.revoria-console-theme :deep(.console-layout .relative.inline-block > span),
.revoria-console-theme :deep(.console-layout .v-popper__inner) {
	border-color: color-mix(in srgb, var(--glass-border) 82%, transparent) !important;
	background: color-mix(in srgb, var(--color-button-bg) 92%, var(--color-raised-bg) 8%) !important;
	color: var(--color-contrast) !important;
}

.revoria-console-theme :deep(.console-layout input) {
	color: var(--color-contrast) !important;
}

.revoria-console-theme :deep(.console-filter-pill) {
	border-color: color-mix(in srgb, var(--glass-border) 82%, transparent) !important;
	background: color-mix(in srgb, var(--color-button-bg) 88%, var(--color-raised-bg) 12%) !important;
	color: var(--color-secondary) !important;
}

.revoria-console-theme :deep(.console-filter-pill:hover) {
	background: color-mix(in srgb, var(--color-button-bg-hover) 86%, transparent) !important;
	color: var(--color-contrast) !important;
}

.revoria-console-theme :deep(.console-filter-pill--active) {
	border-color: color-mix(in srgb, var(--color-button-text-selected) 24%, var(--glass-border)) !important;
	background: color-mix(in srgb, var(--color-button-bg-selected) 88%, var(--color-raised-bg) 12%) !important;
	color: var(--color-button-text-selected) !important;
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-button-text-selected) 18%, transparent);
}

.revoria-console-theme :deep(.console-action-buttons .btn-wrapper) {
	--_bg: color-mix(in srgb, var(--color-button-bg) 88%, var(--color-raised-bg) 12%);
	--_text: var(--color-base);
	--_hover-bg: color-mix(in srgb, var(--color-button-bg-hover) 84%, transparent);
	--_hover-text: var(--color-contrast);
}

.revoria-console-theme :deep(.console-layout .btn-wrapper[color='red']),
.revoria-console-theme :deep(.console-layout .btn-wrapper .text-red) {
	--_text: var(--color-red);
	--_hover-text: var(--color-red);
}

.revoria-console-theme :deep(.console-action-buttons button) {
	white-space: nowrap;
}

.revoria-console-theme :deep(.xterm),
.revoria-console-theme :deep(.xterm-viewport),
.revoria-console-theme :deep(.xterm-screen),
.revoria-console-theme :deep(.xterm .xterm-screen) {
	background: color-mix(in srgb, var(--color-raised-bg) 96%, transparent) !important;
}

.revoria-console-theme :deep(.xterm-scrollable-element > .scrollbar.vertical > div) {
	background: color-mix(in srgb, var(--color-button-bg-selected) 56%, var(--surface-5)) !important;
}

.revoria-console-theme :deep(.btn-wrapper[type='highlight']),
.revoria-console-theme :deep(.btn-wrapper) {
	color-scheme: inherit;
}

.crash-select :deep(.dropdown) {
	background: color-mix(in srgb, var(--color-button-bg) 92%, var(--color-raised-bg) 8%) !important;
}

@media (max-width: 960px) {
	.crash-grid {
		grid-template-columns: 1fr;
	}
}
</style>
