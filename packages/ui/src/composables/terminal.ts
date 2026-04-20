import type { FitAddon } from '@xterm/addon-fit'
import type { SearchAddon } from '@xterm/addon-search'
import type { ITerminalOptions, Terminal } from '@xterm/xterm'
import {
	nextTick,
	onBeforeUnmount,
	onMounted,
	ref,
	shallowRef,
	type Ref,
	type ShallowRef,
} from 'vue'

export function getCssVar(name: string, fallback: string): string {
	if (typeof document === 'undefined') return fallback
	const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
	return value || fallback
}

function buildTerminalTheme() {
	const surface2 = getCssVar('--surface-2', '#1d1f23')
	const surface5 = getCssVar('--surface-5', '#42444a')
	const textDefault = getCssVar('--color-text-default', '#b0bac5')
	const textTertiary = getCssVar('--color-text-tertiary', '#96a2b0')
	const textPrimary = getCssVar('--color-text-primary', '#ffffff')
	const brand = getCssVar('--color-brand', '#1bd96a')
	const red = getCssVar('--color-red', '#ff496e')
	const orange = getCssVar('--color-orange', '#ffa347')
	const blue = getCssVar('--color-blue', '#4a9eff')
	const purple = getCssVar('--color-purple', '#bc3fbc')

	return {
		background: surface2,
		foreground: textDefault,
		cursor: textDefault,
		cursorAccent: surface2,
		selectionBackground: 'rgba(128, 128, 128, 0.3)',
		black: surface2,
		red,
		green: brand,
		yellow: orange,
		blue,
		magenta: purple,
		cyan: textTertiary,
		white: textDefault,
		brightBlack: surface5,
		brightRed: red,
		brightGreen: brand,
		brightYellow: orange,
		brightBlue: blue,
		brightMagenta: purple,
		brightCyan: textTertiary,
		brightWhite: textPrimary,
		scrollbarSliderBackground: surface5,
		scrollbarSliderHoverBackground: surface5,
		scrollbarSliderActiveBackground: surface5,
		overviewRulerBorder: 'transparent',
	}
}

export interface UseTerminalOptions {
	container: Ref<HTMLElement | null>
	options?: ITerminalOptions
	scrollback?: number
	onReady?: (terminal: Terminal) => void
	onResize?: () => void
}

export interface UseTerminalReturn {
	terminal: ShallowRef<Terminal | null>
	fitAddon: ShallowRef<FitAddon | null>
	searchAddon: ShallowRef<SearchAddon | null>
	isAtBottom: Ref<boolean>
	write: (data: string) => void
	writeln: (data: string) => void
	clear: () => void
	reset: () => void
	fit: () => void
	scrollToBottom: () => void
}

export function useTerminal(options: UseTerminalOptions): UseTerminalReturn {
	const terminal = shallowRef<Terminal | null>(null)
	const fitAddon = shallowRef<FitAddon | null>(null)
	const searchAddon = shallowRef<SearchAddon | null>(null)
	const isAtBottom = ref(true)

	let resizeObserver: ResizeObserver | null = null
	let themeObserver: MutationObserver | null = null
	let wheelHandler: ((e: WheelEvent) => void) | null = null
	let hasWritten = false
	const pendingWrites: Array<{ data: string; newline: boolean }> = []

	const write = (data: string) => {
		if (terminal.value) {
			terminal.value.write(data)
			hasWritten = true
		} else {
			pendingWrites.push({ data, newline: false })
		}
	}

	const writeln = (data: string) => {
		if (terminal.value) {
			if (hasWritten) {
				terminal.value.write('\r\n' + data)
			} else {
				terminal.value.write(data)
				hasWritten = true
			}
		} else {
			pendingWrites.push({ data, newline: true })
		}
	}

	const clear = () => {
		terminal.value?.clear()
		hasWritten = false
	}

	const reset = () => {
		terminal.value?.reset()
		hasWritten = false
	}

	const fit = () => {
		const fit = fitAddon.value
		const terminalValue = terminal.value
		if (!fit || !terminalValue) return
		const dims = fit.proposeDimensions()
		if (dims) {
			terminalValue.resize(dims.cols, dims.rows)
		}
	}

	const scrollToBottom = () => {
		terminal.value?.scrollToBottom()
		isAtBottom.value = true

		let calls = 0
		const interval = setInterval(() => {
			terminal.value?.scrollToBottom()
			if (++calls >= 10) clearInterval(interval)
		}, 25)
	}

	const checkIfAtBottom = () => {
		const terminalValue = terminal.value
		if (!terminalValue) return
		const buffer = terminalValue.buffer.active
		isAtBottom.value = buffer.baseY - buffer.viewportY <= 2
	}

	onMounted(async () => {
		const container = options.container.value
		if (!container) return

		const [{ Terminal }, { FitAddon }, { SearchAddon }] = await Promise.all([
			import('@xterm/xterm'),
			import('@xterm/addon-fit'),
			import('@xterm/addon-search'),
		])

		await import('@xterm/xterm/css/xterm.css')

		const terminalValue = new Terminal({
			disableStdin: true,
			scrollback: options.scrollback ?? Infinity,
			convertEol: true,
			smoothScrollDuration: 125,
			fontFamily: 'monospace',
			fontSize: 14,
			lineHeight: 1.5,
			allowProposedApi: true,
			theme: buildTerminalTheme(),
			...options.options,
		})

		const fit = new FitAddon()
		const search = new SearchAddon()

		terminalValue.loadAddon(fit)
		terminalValue.loadAddon(search)
		terminalValue.open(container)
		await nextTick()
		const dims = fit.proposeDimensions()
		if (dims) {
			terminalValue.resize(dims.cols, dims.rows)
		}

		terminalValue.options.disableStdin = true
		terminalValue.write('\x1b[?25l')

		wheelHandler = (e: WheelEvent) => {
			e.preventDefault()
		}
		container.addEventListener('wheel', wheelHandler, { passive: false })

		terminalValue.onScroll(() => checkIfAtBottom())
		terminalValue.onWriteParsed(() => {
			if (isAtBottom.value) {
				terminalValue.scrollToBottom()
			}
		})

		terminal.value = terminalValue
		fitAddon.value = fit
		searchAddon.value = search

		for (const pending of pendingWrites) {
			if (pending.newline) {
				writeln(pending.data)
			} else {
				write(pending.data)
			}
		}
		pendingWrites.length = 0

		resizeObserver = new ResizeObserver(() => {
			const proposed = fit.proposeDimensions()
			if (proposed) {
				terminalValue.resize(proposed.cols, proposed.rows)
			}
			options.onResize?.()
		})
		resizeObserver.observe(container)

		themeObserver = new MutationObserver(() => {
			terminalValue.options.theme = buildTerminalTheme()
		})
		themeObserver.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['data-theme', 'class'],
		})

		options.onReady?.(terminalValue)
	})

	onBeforeUnmount(() => {
		if (wheelHandler && options.container.value) {
			options.container.value.removeEventListener('wheel', wheelHandler)
			wheelHandler = null
		}
		resizeObserver?.disconnect()
		resizeObserver = null
		themeObserver?.disconnect()
		themeObserver = null
		terminal.value?.dispose()
		terminal.value = null
	})

	return {
		terminal,
		fitAddon,
		searchAddon,
		isAtBottom,
		write,
		writeln,
		clear,
		reset,
		fit,
		scrollToBottom,
	}
}
