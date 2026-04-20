import { ref } from 'vue'

import type { Log4jEvent, LogLine } from '../layouts/shared/console/types'

const ERROR_TRIGGERS = ['/ERROR', 'Exception:', ':?]', 'Error', '[thread', '\tat']

function detectLogLevel(lineText: string): LogLine['level'] {
	if (lineText.includes('/INFO') || lineText.includes('[System] [CHAT]')) return 'info'
	if (lineText.includes('/WARN')) return 'warn'
	if (lineText.includes('/DEBUG')) return 'debug'
	if (lineText.includes('/TRACE')) return 'trace'
	for (const trigger of ERROR_TRIGGERS) {
		if (lineText.includes(trigger)) return 'error'
	}
	return null
}

function toLogLines(text: string): LogLine[] {
	return text
		.replace(/\r\n/g, '\n')
		.split('\n')
		.filter((line, index, lines) => line.length > 0 || index < lines.length - 1)
		.map((line) => ({
			text: line,
			level: detectLogLevel(line),
		}))
}

function formatLog4jEvent(event: Log4jEvent): string {
	const timestamp = event.timestamp_millis
		? new Date(event.timestamp_millis).toLocaleTimeString([], { hour12: false })
		: '--:--:--'
	const level = (event.level ?? 'INFO').toUpperCase()
	const thread = event.thread_name ?? 'main'
	const logger = event.logger_name ?? 'Minecraft'
	const message = event.message ?? ''
	return `[${timestamp}] [${thread}/${level}] [${logger}]: ${message}`
}

export function createConsoleState() {
	const output = ref<LogLine[]>([])

	function clear() {
		output.value = []
	}

	function append(lines: LogLine[]) {
		if (lines.length === 0) return
		output.value = [...output.value, ...lines]
	}

	function addLegacyLog(text: string) {
		append(toLogLines(text))
	}

	function addLog4jEvent(event: Log4jEvent) {
		append(toLogLines(formatLog4jEvent(event)))
		if (event.throwable) {
			append(toLogLines(event.throwable))
		}
	}

	return {
		output,
		clear,
		addLegacyLog,
		addLog4jEvent,
	}
}
