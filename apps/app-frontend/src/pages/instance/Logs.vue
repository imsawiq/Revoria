<template>
	<div class="logs-page">
		<Card class="log-card">
			<div class="button-row">
				<DropdownSelect
					v-model="selectedLogIndex"
					:default-value="0"
					:name="formatMessage(messages.logDateLabel)"
					class="log-select"
					:options="logs.map((_, index) => index)"
					:display-name="(option) => logs[option]?.name"
					:disabled="logs.length === 0"
				/>
				<div class="button-group">
					<Button :disabled="!logs[selectedLogIndex]" @click="copyLog()">
						<ClipboardCopyIcon v-if="!copied" />
						<CheckIcon v-else />
						{{ copied ? formatMessage(messages.copied) : formatMessage(messages.copy) }}
					</Button>
					<Button :disabled="!props.instance?.path" @click="openLogsFolder()">
						<FolderOpenIcon aria-hidden="true" />
						{{ formatMessage(messages.openFolder) }}
					</Button>
					<Button color="primary" :disabled="offline || !logs[selectedLogIndex]" @click="share">
						<ShareIcon aria-hidden="true" />
						{{ formatMessage(messages.share) }}
					</Button>
					<Button
						v-if="logs[selectedLogIndex] && logs[selectedLogIndex].live === true"
						@click="clearLiveLog()"
					>
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.clear) }}
					</Button>

					<Button
						v-else
						:disabled="!logs[selectedLogIndex] || logs[selectedLogIndex].live === true"
						color="danger"
						@click="deleteLog()"
					>
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.delete) }}
					</Button>
				</div>
			</div>
			<div class="button-row">
				<input
					id="text-filter"
					v-model="searchFilter"
					autocomplete="off"
					type="text"
					class="text-filter"
					:placeholder="formatMessage(messages.filterPlaceholder)"
				/>
				<div class="filter-group">
					<Checkbox
						v-for="level in levels"
						:key="level.toLowerCase()"
						v-model="levelFilters[level.toLowerCase()]"
						class="filter-checkbox"
					>
						{{ levelLabels[level.toLowerCase()] ?? level }}
					</Checkbox>
				</div>
			</div>
			<div class="log-text">
				<RecycleScroller
					v-slot="{ item }"
					ref="logContainer"
					class="scroller"
					:items="displayProcessedLogs"
					direction="vertical"
					:item-size="20"
					key-field="id"
					:buffer="200"
				>
					<div class="user no-wrap">
						<span :style="{ color: item.prefixColor, 'font-weight': item.weight }">{{
							item.prefix
						}}</span>
						<span :style="{ color: item.textColor }">{{ item.text }}</span>
					</div>
				</RecycleScroller>
			</div>
			<ShareModalWrapper
				ref="shareModal"
				:header="formatMessage(messages.shareLogHeader)"
				:share-title="formatMessage(messages.shareTitle)"
				:share-text="formatMessage(messages.shareText)"
				:open-in-new-tab="false"
				:on-hide="onModalHide"
				link
			/>
		</Card>

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
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

import {
	CheckIcon,
	ClipboardCopyIcon,
	FolderOpenIcon,
	ShareIcon,
	TrashIcon,
} from '@modrinth/assets'
import { Button, Card, Checkbox, DropdownSelect, injectNotificationManager } from '@modrinth/ui'
import { formatBytes, renderString } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import isToday from 'dayjs/plugin/isToday'
import isYesterday from 'dayjs/plugin/isYesterday'
import { ofetch } from 'ofetch'
import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { RecycleScroller } from 'vue-virtual-scroller'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { process_listener } from '@/helpers/events.js'
import {
	delete_logs_by_filename,
	get_latest_log_cursor,
	get_logs,
	get_output_by_filename,
} from '@/helpers/logs.js'
import { get_by_profile_path } from '@/helpers/process.js'
import { get_full_path } from '@/helpers/profile'
import { openPath } from '@/helpers/utils.js'

dayjs.extend(isToday)
dayjs.extend(isYesterday)

const { handleError } = injectNotificationManager()
const route = useRoute()

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
		required: true,
	},
	installed: {
		type: Boolean,
		default() {
			return false
		},
	},
})

const currentLiveLog = ref(null)
const currentLiveLogCursor = ref(0)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	logDateLabel: {
		id: 'instance.logs.log-date',
		defaultMessage: 'Log date',
	},
	copy: {
		id: 'instance.logs.copy',
		defaultMessage: 'Copy',
	},
	copied: {
		id: 'instance.logs.copied',
		defaultMessage: 'Copied',
	},
	openFolder: {
		id: 'instance.logs.open-folder',
		defaultMessage: 'Open folder',
	},
	share: {
		id: 'instance.logs.share',
		defaultMessage: 'Share',
	},
	clear: {
		id: 'instance.logs.clear',
		defaultMessage: 'Clear',
	},
	delete: {
		id: 'instance.logs.delete',
		defaultMessage: 'Delete',
	},
	filterPlaceholder: {
		id: 'instance.logs.filter.placeholder',
		defaultMessage: 'Type to filter logs...',
	},
	shareLogHeader: {
		id: 'instance.logs.share.header',
		defaultMessage: 'Share Log',
	},
	shareTitle: {
		id: 'instance.logs.share.title',
		defaultMessage: 'Instance Log',
	},
	shareText: {
		id: 'instance.logs.share.text',
		defaultMessage: 'Check out this log from an instance on the Modrinth App',
	},
	endpointLabel: {
		id: 'instance.logs.crash.endpoint',
		defaultMessage: 'Endpoint',
	},
	crashLogLabel: {
		id: 'instance.logs.crash.dropdown',
		defaultMessage: 'Crash log',
	},
	noLiveGame: {
		id: 'instance.logs.empty.title',
		defaultMessage: 'No live game detected.',
	},
	startGame: {
		id: 'instance.logs.empty.subtitle',
		defaultMessage: 'Start your game to proceed.',
	},
	liveLog: {
		id: 'instance.logs.live',
		defaultMessage: 'Live Log',
	},
	unknownLog: {
		id: 'instance.logs.unknown',
		defaultMessage: 'Unknown',
	},
	loading: {
		id: 'instance.logs.loading',
		defaultMessage: 'Loading...',
	},
	levelComment: {
		id: 'instance.logs.level.comment',
		defaultMessage: 'Comment',
	},
	levelError: {
		id: 'instance.logs.level.error',
		defaultMessage: 'Error',
	},
	levelWarn: {
		id: 'instance.logs.level.warn',
		defaultMessage: 'Warn',
	},
	levelInfo: {
		id: 'instance.logs.level.info',
		defaultMessage: 'Info',
	},
	levelDebug: {
		id: 'instance.logs.level.debug',
		defaultMessage: 'Debug',
	},
	levelTrace: {
		id: 'instance.logs.level.trace',
		defaultMessage: 'Trace',
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
})
const emptyText = computed(() => [
	formatMessage(messages.noLiveGame),
	formatMessage(messages.startGame),
])

const logs = ref([])

const logsColored = true

const selectedLogIndex = ref(0)
const copied = ref(false)
const logContainer = ref(null)
const interval = ref(null)
const userScrolled = ref(false)
const isAutoScrolling = ref(false)
const shareModal = ref(null)
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
const levelLabels = computed(() => ({
	comment: formatMessage(messages.levelComment),
	error: formatMessage(messages.levelError),
	warn: formatMessage(messages.levelWarn),
	info: formatMessage(messages.levelInfo),
	debug: formatMessage(messages.levelDebug),
	trace: formatMessage(messages.levelTrace),
}))
const onModalHide = () => {
	copied.value = false
}

const openLogsFolder = async () => {
	if (!props.instance?.path) return
	const fullPath = await get_full_path(props.instance.path).catch(handleError)
	if (!fullPath) return
	await openPath(`${fullPath}/logs`).catch(handleError)
}

const openCrashFolder = async () => {
	if (!props.instance?.path) return
	const fullPath = await get_full_path(props.instance.path).catch(handleError)
	if (!fullPath) return
	await openPath(`${fullPath}/crash-reports`).catch(handleError)
}

const levels = ['Comment', 'Error', 'Warn', 'Info', 'Debug', 'Trace']
const levelFilters = ref({})
levels.forEach((level) => {
	levelFilters.value[level.toLowerCase()] = true
})
const searchFilter = ref('')

function shouldDisplay(processedLine) {
	if (!processedLine.level) {
		return true
	}

	if (!levelFilters.value[processedLine.level.toLowerCase()]) {
		return false
	}
	if (searchFilter.value !== '') {
		if (!processedLine.text.toLowerCase().includes(searchFilter.value.toLowerCase())) {
			return false
		}
	}
	return true
}

// Selects from the processed logs which ones should be displayed (shouldDisplay)
// In addition, splits each line by \n. Each split line is given the same properties as the original line
const displayProcessedLogs = computed(() => {
	return processedLogs.value.filter((l) => shouldDisplay(l))
})

const processedLogs = computed(() => {
	// split based on newline and timestamp lookahead
	// (not just newline because of multiline messages)
	const splitPattern = /\n(?=(?:#|\[\d\d:\d\d:\d\d\]))/

	const lines = logs.value[selectedLogIndex.value]?.stdout.split(splitPattern) || []
	const processed = []
	let id = 0
	for (let i = 0; i < lines.length; i++) {
		// Then split off of \n.
		// Lines that are not the first have prefix = null
		const text = getLineText(lines[i])
		const prefix = getLinePrefix(lines[i])
		const prefixColor = getLineColor(lines[i], true)
		const textColor = getLineColor(lines[i], false)
		const weight = getLineWeight(lines[i])
		const level = getLineLevel(lines[i])
		text.split('\n').forEach((line, index) => {
			processed.push({
				id: id,
				text: line,
				prefix: index === 0 ? prefix : null,
				prefixColor: prefixColor,
				textColor: textColor,
				weight: weight,
				level: level,
			})
			id += 1
		})
	}
	return processed
})

async function getLiveStdLog() {
	if (route.params.id && props.instance?.path) {
		const processes = await get_by_profile_path(route.params.id).catch(handleError)
		let returnValue
		if (processes.length === 0) {
			returnValue = emptyText.value.join('\n')
		} else {
			const logCursor = await get_latest_log_cursor(
				props.instance.path,
				currentLiveLogCursor.value,
			).catch(handleError)
			if (logCursor.new_file) {
				currentLiveLog.value = ''
			}
			currentLiveLog.value = currentLiveLog.value + logCursor.output
			currentLiveLogCursor.value = logCursor.cursor
			returnValue = currentLiveLog.value
		}
		return { name: formatMessage(messages.liveLog), stdout: returnValue, live: true }
	}
	return null
}

async function getLogs() {
	if (!props.instance?.path) return []
	return (await get_logs(props.instance.path, true).catch(handleError))
		.filter(
			// filter out latest_stdout.log or anything without .log in it
			(log) =>
				log.filename !== 'latest_stdout.log' &&
				log.filename !== 'latest_stdout' &&
				log.stdout !== '' &&
				(log.filename.includes('.log') || log.filename.endsWith('.txt')),
		)
		.map((log) => {
			log.name = log.filename || formatMessage(messages.unknownLog)
			log.stdout = formatMessage(messages.loading)
			return log
		})
}

async function setLogs() {
	const [liveStd, allLogs] = await Promise.all([getLiveStdLog(), getLogs()])
	logs.value = [liveStd, ...allLogs]
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
			data?.message?.content ??
			data?.response ??
			data?.output ??
			raw ??
			crashText.value.fail
		crashModal.value?.show()
	} catch (error) {
		handleError(error)
	} finally {
		crashLoading.value = false
	}
}

const copyLog = () => {
	if (logs.value.length > 0 && logs.value[selectedLogIndex.value]) {
		navigator.clipboard.writeText(logs.value[selectedLogIndex.value].stdout)
		copied.value = true
	}
}

const share = async () => {
	if (logs.value.length > 0 && logs.value[selectedLogIndex.value]) {
		const url = await ofetch('https://api.mclo.gs/1/log', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/x-www-form-urlencoded',
			},
			body: `content=${encodeURIComponent(logs.value[selectedLogIndex.value].stdout)}`,
		}).catch(handleError)

		shareModal.value.show(url.url)
	}
}

watch(selectedLogIndex, async (newIndex) => {
	copied.value = false
	userScrolled.value = false

	if (logs.value.length > 1 && newIndex !== 0) {
		logs.value[newIndex].stdout = 'Loading...'
		logs.value[newIndex].stdout = await get_output_by_filename(
			props.instance.path,
			logs.value[newIndex].log_type,
			logs.value[newIndex].filename,
		).catch(handleError)
	}
})

if (logs.value.length > 1 && !props.playing) {
	selectedLogIndex.value = 1
} else {
	selectedLogIndex.value = 0
}

const deleteLog = async () => {
	if (logs.value[selectedLogIndex.value] && selectedLogIndex.value !== 0) {
		const deleteIndex = selectedLogIndex.value
		selectedLogIndex.value = deleteIndex - 1
		await delete_logs_by_filename(
			props.instance.path,
			logs.value[deleteIndex].log_type,
			logs.value[deleteIndex].filename,
		).catch(handleError)
		await setLogs()
	}
}

const clearLiveLog = async () => {
	currentLiveLog.value = ''
	// does not reset cursor
}

const isLineLevel = (text, level) => {
	if ((text.includes('/INFO') || text.includes('[System] [CHAT]')) && level === 'info') {
		return true
	}

	if (text.includes('/WARN') && level === 'warn') {
		return true
	}

	if (text.includes('/DEBUG') && level === 'debug') {
		return true
	}

	if (text.includes('/TRACE') && level === 'trace') {
		return true
	}

	const errorTriggers = ['/ERROR', 'Exception:', ':?]', 'Error', '[thread', '	at']
	if (level === 'error') {
		for (const trigger of errorTriggers) {
			if (text.includes(trigger)) return true
		}
	}

	if (text.trim()[0] === '#' && level === 'comment') {
		return true
	}
	return false
}

const getLineWeight = (text) => {
	if (
		!logsColored ||
		isLineLevel(text, 'info') ||
		isLineLevel(text, 'debug') ||
		isLineLevel(text, 'trace')
	) {
		return 'normal'
	}

	if (isLineLevel(text, 'error') || isLineLevel(text, 'warn')) {
		return 'bold'
	}
}

const getLineLevel = (text) => {
	for (const level of levels) {
		if (isLineLevel(text, level.toLowerCase())) {
			return level
		}
	}
}

const getLineColor = (text, prefix) => {
	if (isLineLevel(text, 'comment')) {
		return 'var(--color-green)'
	}

	if (!logsColored || text.includes('[System] [CHAT]')) {
		return 'var(--color-white)'
	}
	if (
		(isLineLevel(text, 'info') || isLineLevel(text, 'debug') || isLineLevel(text, 'trace')) &&
		prefix
	) {
		return 'var(--color-blue)'
	}
	if (isLineLevel(text, 'warn')) {
		return 'var(--color-orange)'
	}
	if (isLineLevel(text, 'error')) {
		return 'var(--color-red)'
	}
}

const getLinePrefix = (text) => {
	if (text.includes(']:')) {
		return text.split(']:')[0] + ']:'
	}
}

const getLineText = (text) => {
	if (text.includes(']:')) {
		if (text.split(']:').length > 2) {
			return text.split(']:').slice(1).join(']:')
		}
		return text.split(']:')[1]
	} else {
		return text
	}
}

function handleUserScroll() {
	if (!isAutoScrolling.value) {
		userScrolled.value = true
	}
}

interval.value = setInterval(async () => {
	if (logs.value.length > 0) {
		logs.value[0] = await getLiveStdLog()
		const scroll = logContainer.value.getScroll()

		// Allow resetting of userScrolled if the user scrolls to the bottom
		if (selectedLogIndex.value === 0) {
			if (scroll.end >= logContainer.value.$el.scrollHeight - 10) userScrolled.value = false
			if (!userScrolled.value) {
				await nextTick()
				isAutoScrolling.value = true
				logContainer.value.scrollToItem(displayProcessedLogs.value.length - 1)
				setTimeout(() => (isAutoScrolling.value = false), 50)
			}
		}
	}
}, 250)

let unlistenProcesses = null

onMounted(async () => {
	await setLogs()
	await loadCrashLogs()
	logContainer.value?.$el?.addEventListener('scroll', handleUserScroll)

	unlistenProcesses = await process_listener(async (e) => {
		if (e.event === 'launched') {
			currentLiveLog.value = ''
			currentLiveLogCursor.value = 0
			selectedLogIndex.value = 0
		}
		if (e.event === 'finished') {
			currentLiveLog.value = ''
			currentLiveLogCursor.value = 0
			userScrolled.value = false
			await setLogs()
			selectedLogIndex.value = 1
		}
	}).catch(() => null)
})

onBeforeUnmount(() => {
	logContainer.value?.$el?.removeEventListener('scroll', handleUserScroll)
})
onUnmounted(() => {
	clearInterval(interval.value)
	unlistenProcesses?.()
})
</script>

<style scoped lang="scss">
.logs-page {
	display: flex;
	flex-direction: column;
	gap: 1.5rem;
}

.log-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	min-height: 60vh;
	min-width: 0;
}

.log-select {
	max-width: 18rem;
	border-radius: 999px;
	background: var(--color-raised-bg);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
}

.crash-card {
	padding: 1.5rem;
	display: flex;
	flex-direction: column;
	gap: 1.25rem;
}

.crash-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
}

.crash-grid {
	display: grid;
	grid-template-columns: minmax(220px, 1fr) minmax(320px, 2fr);
	gap: 1.5rem;
}

.crash-settings {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.crash-input {
	width: 100%;
	background: var(--color-raised-bg);
	border: 1px solid var(--glass-border);
	border-radius: 0.75rem;
	padding: 0.5rem 0.75rem;
	color: var(--color-contrast);
}

.crash-logs {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.crash-meta {
	padding: 0.5rem 0.75rem;
	border-radius: 0.75rem;
	background: var(--color-button-bg);
}

.crash-actions {
	display: flex;
	flex-wrap: wrap;
	gap: 0.5rem;
}

.modal-content {
	max-height: 70vh;
	overflow: auto;
}

.analysis-text {
	margin: 0;
	max-height: 60vh;
	overflow: auto;
}

:deep(.analysis-text.markdown-body) {
	font-family: var(--font-standard);
	line-height: 1.5;
	white-space: normal;
	word-break: break-word;
}

:deep(.analysis-text.markdown-body pre),
:deep(.analysis-text.markdown-body code) {
	font-family: var(--mono-font);
}

:deep(.analysis-text.markdown-body table) {
	display: block;
	max-width: 100%;
	overflow-x: auto;
	border-collapse: collapse;
}

:deep(.analysis-text.markdown-body th),
:deep(.analysis-text.markdown-body td) {
	white-space: nowrap;
}

.crash-select {
	max-width: 20rem;
	border-radius: 999px;
	background: var(--color-raised-bg);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
}

:global(.v-popper__inner) {
	background: var(--color-raised-bg) !important;
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: none !important;
	-webkit-backdrop-filter: none !important;
	color: var(--color-contrast);
}

:global(.v-popper__arrow-inner) {
	border-color: var(--glass-border);
}

.button-row {
	display: flex;
	flex-direction: row;
	justify-content: space-between;
	gap: 0.5rem;
}

.button-group {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
}

.log-text {
	width: 100%;
	flex: 1 1 auto;
	min-height: 18rem;
	height: clamp(20rem, 44vh, 34rem);
	font-family: var(--mono-font);
	background-color: var(--color-accent-contrast);
	color: var(--color-contrast);
	border-radius: var(--radius-lg);
	padding-top: 0.75rem;
	overflow: hidden;
	white-space: normal;
	color-scheme: dark;
}

.filter-checkbox {
	margin-bottom: 0.3rem;
	font-size: 1rem;

	svg {
		display: flex;
		align-self: center;
		justify-self: center;
	}
}

.filter-group {
	display: flex;
	padding: 0.6rem;
	flex-direction: row;
	overflow: auto;
	gap: 0.5rem;

	&::-webkit-scrollbar-track,
	&::-webkit-scrollbar-thumb {
		border-radius: 10px;
	}
}

:deep(.vue-recycle-scroller__item-wrapper) {
	overflow: visible; /* Enables horizontal scrolling */
}

:deep(.vue-recycle-scroller) {
	&::-webkit-scrollbar-corner {
		background-color: var(--color-bg);
		border-radius: 0 0 10px 0;
	}
}

.scroller {
	height: 100%;
	overflow: auto;
}

.user {
	min-height: 20px;
	padding: 0 1.5rem;
	display: flex;
	align-items: flex-start;
	user-select: text;
}
</style>
