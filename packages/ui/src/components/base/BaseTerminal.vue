<template>
	<div
		class="flex w-full flex-col overflow-hidden rounded-[20px] border border-solid border-surface-4 bg-surface-2"
		:style="!fullscreen && componentHeight ? { minHeight: `${componentHeight}px` } : {}"
		:class="{ 'h-full': fullscreen }"
	>
		<div ref="wrapperRef" class="relative min-h-0 flex-1 overflow-hidden pb-2 pt-1">
			<div
				ref="containerRef"
				class="size-full"
				@mouseenter="isPointerOverTerminal = true"
				@mouseleave="isPointerOverTerminal = false"
				@copy.prevent="copyTerminalContent"
			/>
			<div v-if="!isAtBottom" class="absolute bottom-4 right-4 z-10">
				<ButtonStyled circular type="highlight" size="large">
					<button :aria-label="formatMessage(messages.scrollToBottom)" @click="scrollToBottom">
						<ChevronDownIcon />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div
			v-if="showInput"
			ref="inputRef"
			class="border-x-0 border-b-0 border-t border-solid border-surface-4 bg-surface-3 p-4"
		>
			<StyledInput
				v-model="commandInput"
				:icon="TerminalSquareIcon"
				:placeholder="
					disableInput
						? formatMessage(messages.serverNotRunning)
						: formatMessage(messages.sendCommand)
				"
				:disabled="disableInput"
				wrapper-class="w-full"
				input-class="!h-10"
				@keydown.enter="submitCommand"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, TerminalSquareIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'
import type { Terminal } from '@xterm/xterm'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { useTerminal } from '#ui/composables/terminal'

const props = withDefaults(
	defineProps<{
		scrollback?: number
		showInput?: boolean
		disableInput?: boolean
		fullscreen?: boolean
		emptyStateType?: 'server' | 'instance'
	}>(),
	{
		scrollback: Infinity,
		showInput: false,
		disableInput: false,
		fullscreen: false,
		emptyStateType: undefined,
	},
)

const { formatMessage } = useVIntl()
const messages = defineMessages({
	scrollToBottom: {
		id: 'console.base.scroll-to-bottom',
		defaultMessage: 'Scroll to bottom',
	},
	serverNotRunning: {
		id: 'console.base.server-not-running',
		defaultMessage: 'Server is not running',
	},
	sendCommand: {
		id: 'console.base.send-command',
		defaultMessage: 'Send a command',
	},
	serverWelcomeTop: {
		id: 'console.base.server-empty.top',
		defaultMessage: 'Welcome to your Modrinth Server!',
	},
	serverWelcomeBottom: {
		id: 'console.base.server-empty.bottom',
		defaultMessage: 'Press the green start button to start your server!',
	},
	instanceWelcomeTop: {
		id: 'console.base.instance-empty.top',
		defaultMessage: 'Start your instance in the top right to start',
	},
	instanceWelcomeBottom: {
		id: 'console.base.instance-empty.bottom',
		defaultMessage: 'receiving live logs!',
	},
})

const emit = defineEmits<{
	command: [command: string]
	ready: [terminal: Terminal]
}>()

const FROG = [
	'\x1B[32m     _    _ \x1B[37m',
	'\x1B[32m    (o)--(o)      \x1B[37m',
	'\x1B[32m   /.______.\\\\\x1B[37m',
	'\x1B[32m   \\\\________/     \x1B[37m',
	'\x1B[32m  ./        \\\\.    \x1B[37m',
	'\x1B[32m ( .        , )\x1B[37m',
	'\x1B[32m  \\\\ \\\\_\\\\ //_/ /\x1B[37m',
	'\x1B[32m   ~~  ~~  ~~\x1B[37m',
]

const EMPTY_STATE_BUBBLES = {
	server: [
		'   __________________________________________________',
		` /  ${formatMessage(messages.serverWelcomeTop).padEnd(50, ' ')}\\\\`,
		`|   ${formatMessage(messages.serverWelcomeBottom).padEnd(48, ' ')}|`,
		' \\\\____________________________________________________/',
	],
	instance: [
		'   _____________________________________________________________',
		` /  ${formatMessage(messages.instanceWelcomeTop).padEnd(57, ' ')}\\\\`,
		`|   ${formatMessage(messages.instanceWelcomeBottom).padEnd(57, ' ')}|`,
		' \\\\_____________________________________________________________/',
	],
}

const containerRef = ref<HTMLElement | null>(null)
const wrapperRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLElement | null>(null)
const commandInput = ref('')
const componentHeight = ref(0)
const snappedHeight = ref<number | null>(null)
const showingEmptyState = ref(false)
const isPointerOverTerminal = ref(false)
const isTerminalFocused = ref(false)
let copyHandler: ((event: KeyboardEvent) => void) | null = null
let focusHandler: (() => void) | null = null
let blurHandler: (() => void) | null = null

const {
	terminal,
	searchAddon,
	isAtBottom,
	write,
	writeln,
	clear,
	reset,
	fit: rawFit,
	scrollToBottom,
} = useTerminal({
	container: containerRef,
	scrollback: props.scrollback,
	onReady: (terminalValue) => {
		nextTick(() => {
			updateComponentHeight()
			snapToRows()
		})
		emit('ready', terminalValue)
	},
	onResize: () => {
		updateComponentHeight()
	},
})

function writeEmptyState() {
	if (!terminal.value || !props.emptyStateType) return
	terminal.value.reset()
	const bubble = EMPTY_STATE_BUBBLES[props.emptyStateType]
	if (bubble) {
		for (const line of [...bubble, ...FROG]) {
			terminal.value.writeln(line)
		}
	}
	showingEmptyState.value = true
}

function clearEmptyState() {
	if (!showingEmptyState.value) return
	terminal.value?.reset()
	showingEmptyState.value = false
}

function getWrapperMargins() {
	if (!wrapperRef.value) return 0
	const style = getComputedStyle(wrapperRef.value)
	return parseFloat(style.marginTop) + parseFloat(style.marginBottom)
}

function snapToRows() {
	if (!props.fullscreen) {
		snappedHeight.value = null
		return
	}
	const screen = containerRef.value?.querySelector('.xterm-screen') as HTMLElement | null
	if (!screen) {
		snappedHeight.value = null
		return
	}
	const inputHeight = inputRef.value?.offsetHeight ?? 0
	const borderWidth = 2
	snappedHeight.value = screen.offsetHeight + getWrapperMargins() + inputHeight + borderWidth
}

let resizeDebounce: ReturnType<typeof setTimeout> | null = null

function handleWindowResize() {
	if (!props.fullscreen) return
	if (resizeDebounce) clearTimeout(resizeDebounce)
	snappedHeight.value = null
	resizeDebounce = setTimeout(() => {
		rawFit()
		nextTick(() => snapToRows())
	}, 50)
}

function getBufferContent() {
	const terminalValue = terminal.value
	if (!terminalValue) return ''
	const buffer = terminalValue.buffer.active
	const lines: string[] = []
	for (let index = 0; index < buffer.length; index++) {
		lines.push(buffer.getLine(index)?.translateToString(true) ?? '')
	}
	return lines.join('\n').trimEnd()
}

async function copyTerminalContent() {
	const terminalValue = terminal.value
	if (!terminalValue) return
	const selectedText = terminalValue.getSelection()
	const content = selectedText || getBufferContent()
	if (!content) return
	if (navigator.clipboard?.writeText) {
		await navigator.clipboard.writeText(content)
		return
	}
	const textarea = document.createElement('textarea')
	textarea.value = content
	textarea.style.position = 'fixed'
	textarea.style.left = '-9999px'
	textarea.style.top = '-9999px'
	document.body.appendChild(textarea)
	textarea.focus()
	textarea.select()
	document.execCommand('copy')
	textarea.remove()
}

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
	copyHandler = (event) => {
		if (!(event.ctrlKey || event.metaKey) || event.code !== 'KeyC') return
		if (!isPointerOverTerminal.value && !isTerminalFocused.value && !terminal.value?.hasSelection())
			return
		event.preventDefault()
		event.stopPropagation()
		void copyTerminalContent()
	}
	document.addEventListener('keydown', copyHandler, true)
	nextTick(() => {
		const textarea = containerRef.value?.querySelector(
			'.xterm-helper-textarea',
		) as HTMLElement | null
		if (!textarea) return
		focusHandler = () => {
			isTerminalFocused.value = true
		}
		blurHandler = () => {
			isTerminalFocused.value = false
		}
		textarea.addEventListener('focus', focusHandler)
		textarea.addEventListener('blur', blurHandler)
	})
})

onBeforeUnmount(() => {
	window.removeEventListener('resize', handleWindowResize)
	if (copyHandler) {
		document.removeEventListener('keydown', copyHandler, true)
		copyHandler = null
	}
	const textarea = containerRef.value?.querySelector('.xterm-helper-textarea') as HTMLElement | null
	if (textarea && focusHandler) textarea.removeEventListener('focus', focusHandler)
	if (textarea && blurHandler) textarea.removeEventListener('blur', blurHandler)
	focusHandler = null
	blurHandler = null
	if (resizeDebounce) clearTimeout(resizeDebounce)
})

function fit() {
	rawFit()
	snapToRows()
}

watch(
	() => props.fullscreen,
	() => {
		if (props.fullscreen) {
			nextTick(() => {
				rawFit()
				nextTick(() => snapToRows())
			})
		} else {
			snappedHeight.value = null
			componentHeight.value = 0
		}
	},
)

function updateComponentHeight() {
	const screen = containerRef.value?.querySelector('.xterm-screen') as HTMLElement | null
	if (!screen) return
	const screenHeight = screen.offsetHeight
	const inputHeight = inputRef.value?.offsetHeight ?? 0
	const borderWidth = 2
	componentHeight.value = screenHeight + getWrapperMargins() + inputHeight + borderWidth
}

function submitCommand() {
	const cmd = commandInput.value.trim()
	if (!cmd) return
	emit('command', cmd)
	commandInput.value = ''
}

defineExpose({
	write,
	writeln,
	clear,
	reset,
	fit,
	scrollToBottom,
	terminal,
	searchAddon,
	isAtBottom,
	commandInput,
	showingEmptyState,
	writeEmptyState,
	clearEmptyState,
})
</script>

<style>
.xterm {
	height: 100% !important;
}

.xterm,
.xterm * {
	color-scheme: inherit;
}

.xterm-viewport,
.xterm-scrollable-element,
.xterm-screen,
.xterm-helpers,
.xterm-helper-textarea {
	background-color: var(--surface-2) !important;
}

.xterm-screen {
	right: 0 !important;
}

.xterm-viewport {
	right: 0 !important;
	width: 100% !important;
}

.xterm .xterm-screen {
	width: 100%;
	margin-left: 0;
	margin-right: 0;
}

.xterm .xterm-rows {
	position: relative;
	z-index: 7;
}

.xterm .xterm-decoration-container {
	overflow: visible !important;
}

.xterm .xterm-decoration-container > div {
	box-sizing: content-box !important;
	margin-left: -12px !important;
	padding-left: 12px !important;
	padding-right: 12px !important;
}

.xterm-scrollable-element > .scrollbar.vertical {
	width: 8px !important;
	background: transparent !important;
}

.xterm-scrollable-element > .scrollbar.vertical > div {
	width: 6px !important;
	border-radius: 8px !important;
	contain: layout style !important;
	background: color-mix(in srgb, var(--color-button-bg-selected) 42%, transparent) !important;
}
</style>
