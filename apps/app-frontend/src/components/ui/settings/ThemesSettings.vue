<script setup lang="ts">
import {
	CopyIcon,
	DownloadIcon,
	PlusIcon,
	RedoIcon,
	TrashIcon,
	UndoIcon,
	UploadIcon,
} from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref } from 'vue'

import {
	cloneCustomTheme,
	createEmptyCustomTheme,
	customThemeToCssVariables,
	downloadCustomTheme,
	getCustomThemeBaseOptions,
	getCustomThemePreviewClasses,
	normalizeImportedCustomTheme,
	snapshotPaletteFromTheme,
	type CustomTheme,
	type CustomThemeBase,
} from '@/helpers/custom-themes'
import { get, set } from '@/helpers/settings.ts'
import { useTheming } from '@/store/state'
import type { ColorTheme } from '@/store/theme'

const themeStore = useTheming()
const { formatMessage } = useVIntl()
const settings = ref(await get())
const importInput = ref<HTMLInputElement | null>(null)
const selectedPreset = ref<CustomThemeBase>(resolveBaseTheme(themeStore.selectedTheme))
const editorThemeId = ref<string | null>(
	themeStore.getActiveCustomTheme()?.id ?? themeStore.customThemes[0]?.id ?? null,
)
const draftTheme = ref<CustomTheme | null>(null)
const notice = ref('')
const importError = ref('')

const messages = defineMessages({
	title: { id: 'settings.theme-studio.title', defaultMessage: 'Theme studio' },
	description: {
		id: 'settings.theme-studio.description',
		defaultMessage: 'Pick a base preset, create an editable copy, then tune the whole launcher palette clearly.',
	},
	official: { id: 'settings.theme-studio.official', defaultMessage: 'Base presets' },
	custom: { id: 'settings.theme-studio.custom', defaultMessage: 'Your themes' },
	newTheme: { id: 'settings.theme-studio.new-theme', defaultMessage: 'New blank theme' },
	importTheme: { id: 'settings.theme-studio.import-theme', defaultMessage: 'Import theme' },
	empty: {
		id: 'settings.theme-studio.empty',
		defaultMessage: 'No custom themes yet. Pick a preset and create the first editable theme from it.',
	},
	editor: { id: 'settings.theme-studio.editor', defaultMessage: 'Editor' },
	name: { id: 'settings.theme-studio.name', defaultMessage: 'Theme name' },
	author: { id: 'settings.theme-studio.author', defaultMessage: 'Author' },
	baseTheme: { id: 'settings.theme-studio.base-theme', defaultMessage: 'Base preset' },
	descriptionLabel: { id: 'settings.theme-studio.description-label', defaultMessage: 'Description' },
	resetBase: { id: 'settings.theme-studio.reset-base', defaultMessage: 'Reset from base preset' },
	undo: { id: 'settings.theme-studio.undo', defaultMessage: 'Отменить' },
	redo: { id: 'settings.theme-studio.redo', defaultMessage: 'Вернуть' },
	applyTheme: { id: 'settings.theme-studio.apply-theme', defaultMessage: 'Apply theme' },
	duplicate: { id: 'settings.theme-studio.duplicate', defaultMessage: 'Duplicate' },
	exportTheme: { id: 'settings.theme-studio.export-theme', defaultMessage: 'Export' },
	copyJson: { id: 'settings.theme-studio.copy-json', defaultMessage: 'Copy JSON' },
	deleteTheme: { id: 'settings.theme-studio.delete-theme', defaultMessage: 'Delete' },
	active: { id: 'settings.theme-studio.active', defaultMessage: 'Active' },
	importFailed: {
		id: 'settings.theme-studio.import-failed',
		defaultMessage: 'This file is not a valid Revoria theme.',
	},
	saved: { id: 'settings.theme-studio.saved', defaultMessage: 'Theme saved.' },
	copied: { id: 'settings.theme-studio.copied', defaultMessage: 'Theme JSON copied.' },
	createFromPreset: {
		id: 'settings.theme-studio.create-from-preset',
		defaultMessage: 'Create editable copy',
	},
	colorsSection: {
		id: 'settings.theme-studio.section.colors',
		defaultMessage: 'Base surfaces',
	},
	uiSection: {
		id: 'settings.theme-studio.section.ui',
		defaultMessage: 'Buttons and panels',
	},
	accentSection: {
		id: 'settings.theme-studio.section.accent',
		defaultMessage: 'Акцент и фирменные переходы',
	},
	stateSection: {
		id: 'settings.theme-studio.section.state',
		defaultMessage: 'States and accents',
	},
	glassSection: {
		id: 'settings.theme-studio.section.glass',
		defaultMessage: 'Стекло и прозрачность',
	},
	metaSection: {
		id: 'settings.theme-studio.section.meta',
		defaultMessage: 'Theme info',
	},
	canvas: { id: 'settings.theme-studio.canvas', defaultMessage: 'Фон приложения' },
	layer2: { id: 'settings.theme-studio.layer-2', defaultMessage: 'Второй слой' },
	layer3: { id: 'settings.theme-studio.layer-3', defaultMessage: 'Третий слой' },
	cards: { id: 'settings.theme-studio.cards', defaultMessage: 'Карточки и крупные блоки' },
	hoverLayer: { id: 'settings.theme-studio.hover-layer', defaultMessage: 'Слой наведения' },
	raisedBg: { id: 'settings.theme-studio.raised-bg', defaultMessage: 'Фон внутренних панелей' },
	raisedBgHover: { id: 'settings.theme-studio.raised-bg-hover', defaultMessage: 'Фон панелей при наведении' },
	buttonBg: { id: 'settings.theme-studio.button-bg', defaultMessage: 'Обычные кнопки' },
	buttonBgHover: { id: 'settings.theme-studio.button-bg-hover', defaultMessage: 'Кнопки при наведении' },
	buttonBgActive: { id: 'settings.theme-studio.button-bg-active', defaultMessage: 'Нажатые кнопки' },
	buttonBorder: { id: 'settings.theme-studio.button-border', defaultMessage: 'Обводка кнопок и полей' },
	buttonSelected: { id: 'settings.theme-studio.button-selected', defaultMessage: 'Активный выбор' },
	buttonSelectedText: { id: 'settings.theme-studio.button-selected-text', defaultMessage: 'Текст активного выбора' },
	divider: { id: 'settings.theme-studio.divider', defaultMessage: 'Разделители и тонкие линии' },
	textPrimary: { id: 'settings.theme-studio.text-primary', defaultMessage: 'Заголовки' },
	textDefault: { id: 'settings.theme-studio.text-default', defaultMessage: 'Основной текст' },
	textSecondary: { id: 'settings.theme-studio.text-secondary', defaultMessage: 'Вторичный текст' },
	brand: { id: 'settings.theme-studio.brand', defaultMessage: 'Главный акцент темы' },
	brandTintStrength: {
		id: 'settings.theme-studio.brand-tint-strength',
		defaultMessage: 'Общая сила фирменного оттенка',
	},
	brandHighlight: { id: 'settings.theme-studio.brand-highlight', defaultMessage: 'Подсветка акцента' },
	brandHighlightOpacity: {
		id: 'settings.theme-studio.brand-highlight-opacity',
		defaultMessage: 'Прозрачность подсветки акцента',
	},
	brandShadow: { id: 'settings.theme-studio.brand-shadow', defaultMessage: 'Тень акцента' },
	brandShadowOpacity: {
		id: 'settings.theme-studio.brand-shadow-opacity',
		defaultMessage: 'Прозрачность тени акцента',
	},
	brandGradientStart: {
		id: 'settings.theme-studio.brand-gradient-start',
		defaultMessage: 'Мягкий градиент: начало',
	},
	brandGradientStartOpacity: {
		id: 'settings.theme-studio.brand-gradient-start-opacity',
		defaultMessage: 'Прозрачность начала мягкого градиента',
	},
	brandGradientEnd: {
		id: 'settings.theme-studio.brand-gradient-end',
		defaultMessage: 'Мягкий градиент: конец',
	},
	brandGradientEndOpacity: {
		id: 'settings.theme-studio.brand-gradient-end-opacity',
		defaultMessage: 'Прозрачность конца мягкого градиента',
	},
	brandGradientStrongStart: {
		id: 'settings.theme-studio.brand-gradient-strong-start',
		defaultMessage: 'Крупный градиент: фон',
	},
	brandGradientStrongStartOpacity: {
		id: 'settings.theme-studio.brand-gradient-strong-start-opacity',
		defaultMessage: 'Прозрачность фона крупного градиента',
	},
	brandGradientStrongEnd: {
		id: 'settings.theme-studio.brand-gradient-strong-end',
		defaultMessage: 'Крупный градиент: акцент',
	},
	brandGradientStrongEndOpacity: {
		id: 'settings.theme-studio.brand-gradient-strong-end-opacity',
		defaultMessage: 'Прозрачность акцента крупного градиента',
	},
	brandGradientBorderColor: {
		id: 'settings.theme-studio.brand-gradient-border-color',
		defaultMessage: 'Обводка акцентных панелей',
	},
	brandGradientBorderOpacity: {
		id: 'settings.theme-studio.brand-gradient-border-opacity',
		defaultMessage: 'Прозрачность обводки акцентных панелей',
	},
	loadingBarEnd: {
		id: 'settings.theme-studio.loading-bar-end',
		defaultMessage: 'Полоса загрузки: второй цвет',
	},
	success: { id: 'settings.theme-studio.success', defaultMessage: 'Успех и запуск' },
	warning: { id: 'settings.theme-studio.warning', defaultMessage: 'Предупреждения' },
	danger: { id: 'settings.theme-studio.danger', defaultMessage: 'Ошибки и удаление' },
	info: { id: 'settings.theme-studio.info', defaultMessage: 'Информация и ссылки' },
	utility: { id: 'settings.theme-studio.utility', defaultMessage: 'Дополнительный акцент' },
	glassTint: { id: 'settings.theme-studio.glass-tint', defaultMessage: 'Цвет стеклянной подложки' },
	glassBorderColor: { id: 'settings.theme-studio.glass-border-color', defaultMessage: 'Цвет стеклянной обводки' },
	buttonBgOpacity: { id: 'settings.theme-studio.button-bg-opacity', defaultMessage: 'Прозрачность обычных кнопок' },
	buttonBorderOpacity: {
		id: 'settings.theme-studio.button-border-opacity',
		defaultMessage: 'Прозрачность обводки кнопок',
	},
	buttonSelectedOpacity: {
		id: 'settings.theme-studio.button-selected-opacity',
		defaultMessage: 'Прозрачность активного выбора',
	},
	glassOpacity: { id: 'settings.theme-studio.glass-opacity', defaultMessage: 'Прозрачность стекла' },
	glassStrong: { id: 'settings.theme-studio.glass-strong', defaultMessage: 'Прозрачность плотного стекла' },
	glassBorder: { id: 'settings.theme-studio.glass-border', defaultMessage: 'Прозрачность стеклянной обводки' },
})

const undoHistory = ref<CustomTheme[]>([])
const redoHistory = ref<CustomTheme[]>([])
const interactionSnapshot = ref<CustomTheme | null>(null)
const interactionToken = ref<string | null>(null)

const normalizedStoredThemes = themeStore.customThemes
	.map((theme) => normalizeImportedCustomTheme(theme))
	.filter((theme): theme is CustomTheme => theme !== null)

if (JSON.stringify(normalizedStoredThemes) !== JSON.stringify(themeStore.customThemes)) {
	themeStore.saveCustomThemes(normalizedStoredThemes)
}

function resolveBaseTheme(theme: ColorTheme): CustomThemeBase {
	return (theme === 'system' ? 'dark' : theme) as CustomThemeBase
}

function cloneDraftTheme(theme: CustomTheme | null) {
	if (!theme) return null
	return {
		...theme,
		palette: {
			...theme.palette,
		},
	}
}

function setDraft(theme: CustomTheme | null, options: { resetHistory?: boolean } = {}) {
	draftTheme.value = cloneDraftTheme(theme)
	if (theme) selectedPreset.value = theme.baseTheme
	notice.value = ''
	importError.value = ''
	interactionSnapshot.value = null
	interactionToken.value = null
	if (options.resetHistory !== false) {
		undoHistory.value = []
		redoHistory.value = []
	}
}

setDraft(themeStore.customThemes.find((theme) => theme.id === editorThemeId.value) ?? null)

const presetThemes = getCustomThemeBaseOptions()
const coreFields = computed(() => [
	{ key: 'surface1', label: formatMessage(messages.canvas) },
	{ key: 'surface2', label: formatMessage(messages.layer2) },
	{ key: 'surface3', label: formatMessage(messages.layer3) },
	{ key: 'surface4', label: formatMessage(messages.cards) },
	{ key: 'surface5', label: formatMessage(messages.hoverLayer) },
	{ key: 'textPrimary', label: formatMessage(messages.textPrimary) },
	{ key: 'textDefault', label: formatMessage(messages.textDefault) },
	{ key: 'textSecondary', label: formatMessage(messages.textSecondary) },
] as const)

const uiFields = computed(() => [
	{ key: 'raisedBg', label: formatMessage(messages.raisedBg) },
	{ key: 'raisedBgHover', label: formatMessage(messages.raisedBgHover) },
	{ key: 'buttonBg', label: formatMessage(messages.buttonBg) },
	{ key: 'buttonBgHover', label: formatMessage(messages.buttonBgHover) },
	{ key: 'buttonBgActive', label: formatMessage(messages.buttonBgActive) },
	{ key: 'buttonBorder', label: formatMessage(messages.buttonBorder) },
	{ key: 'buttonSelected', label: formatMessage(messages.buttonSelected) },
	{ key: 'buttonSelectedText', label: formatMessage(messages.buttonSelectedText) },
	{ key: 'divider', label: formatMessage(messages.divider) },
] as const)

const uiSliderFields = computed(() => [
	{ key: 'buttonBgOpacity', label: formatMessage(messages.buttonBgOpacity), min: 0, max: 100, suffix: '%' },
	{
		key: 'buttonBorderOpacity',
		label: formatMessage(messages.buttonBorderOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'buttonSelectedOpacity',
		label: formatMessage(messages.buttonSelectedOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
] as const)

const stateFields = computed(() => [
	{ key: 'success', label: formatMessage(messages.success) },
	{ key: 'warning', label: formatMessage(messages.warning) },
	{ key: 'danger', label: formatMessage(messages.danger) },
	{ key: 'info', label: formatMessage(messages.info) },
	{ key: 'utility', label: formatMessage(messages.utility) },
] as const)

const accentColorFields = computed(() => [
	{ key: 'brand', label: formatMessage(messages.brand) },
	{ key: 'brandHighlight', label: formatMessage(messages.brandHighlight) },
	{ key: 'brandShadow', label: formatMessage(messages.brandShadow) },
	{ key: 'brandGradientStart', label: formatMessage(messages.brandGradientStart) },
	{ key: 'brandGradientEnd', label: formatMessage(messages.brandGradientEnd) },
	{ key: 'brandGradientStrongStart', label: formatMessage(messages.brandGradientStrongStart) },
	{ key: 'brandGradientStrongEnd', label: formatMessage(messages.brandGradientStrongEnd) },
	{ key: 'brandGradientBorderColor', label: formatMessage(messages.brandGradientBorderColor) },
	{ key: 'loadingBarEnd', label: formatMessage(messages.loadingBarEnd) },
] as const)

const accentSliderFields = computed(() => [
	{
		key: 'brandTintStrength',
		label: formatMessage(messages.brandTintStrength),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandHighlightOpacity',
		label: formatMessage(messages.brandHighlightOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandShadowOpacity',
		label: formatMessage(messages.brandShadowOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandGradientStartOpacity',
		label: formatMessage(messages.brandGradientStartOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandGradientEndOpacity',
		label: formatMessage(messages.brandGradientEndOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandGradientStrongStartOpacity',
		label: formatMessage(messages.brandGradientStrongStartOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandGradientStrongEndOpacity',
		label: formatMessage(messages.brandGradientStrongEndOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
	{
		key: 'brandGradientBorderOpacity',
		label: formatMessage(messages.brandGradientBorderOpacity),
		min: 0,
		max: 100,
		suffix: '%',
	},
] as const)

const glassColorFields = computed(() => [
	{ key: 'glassTint', label: formatMessage(messages.glassTint) },
	{ key: 'glassBorderColor', label: formatMessage(messages.glassBorderColor) },
] as const)

const glassSliderFields = computed(() => [
	{ key: 'glassOpacity', label: formatMessage(messages.glassOpacity), min: 10, max: 100, suffix: '%' },
	{ key: 'glassStrongOpacity', label: formatMessage(messages.glassStrong), min: 10, max: 100, suffix: '%' },
	{ key: 'glassBorderOpacity', label: formatMessage(messages.glassBorder), min: 0, max: 40, suffix: '%' },
] as const)

function themeLabel(theme: ColorTheme) {
	return formatMessage({ id: `settings.display.theme.${theme}`, defaultMessage: theme })
}

async function saveSettingsTheme(theme: ColorTheme) {
	settings.value.theme = theme
	await set(settings.value)
}

async function pickPreset(theme: CustomThemeBase) {
	selectedPreset.value = theme
	notice.value = ''
	importError.value = ''
	themeStore.clearCustomThemeSelection()
	themeStore.setThemeState(selectedPreset.value)
	await saveSettingsTheme(selectedPreset.value)
}

function createTheme() {
	const theme = createEmptyCustomTheme(selectedPreset.value, snapshotPaletteFromTheme(selectedPreset.value))
	theme.name = `Custom ${themeStore.customThemes.length + 1}`
	themeStore.saveCustomThemes([...themeStore.customThemes, theme])
	editorThemeId.value = theme.id
	setDraft(theme)
	void selectTheme(theme.id)
}

function createBlankTheme() {
	selectedPreset.value = resolveBaseTheme(themeStore.selectedTheme)
	createTheme()
}

async function selectTheme(id: string) {
	editorThemeId.value = id
	const theme = themeStore.customThemes.find((entry) => entry.id === id) ?? null
	setDraft(theme)
	if (!theme) return
	themeStore.activateCustomTheme(theme.id)
	themeStore.syncCustomThemeOverlay()
	await saveSettingsTheme(theme.baseTheme)
}

function persistDraft(nextDraft: CustomTheme, options: { syncBaseTheme?: boolean } = {}) {
	const updatedDraft = {
		...nextDraft,
		updatedAt: new Date().toISOString(),
		palette: { ...nextDraft.palette },
	}

	if (themeStore.activeCustomThemeId === updatedDraft.id && options.syncBaseTheme) {
		themeStore.setThemeState(updatedDraft.baseTheme, { preserveCustom: true })
		void saveSettingsTheme(updatedDraft.baseTheme)
	}

	const nextThemes = themeStore.customThemes.some((theme) => theme.id === updatedDraft.id)
		? themeStore.customThemes.map((theme) => (theme.id === updatedDraft.id ? updatedDraft : theme))
		: [...themeStore.customThemes, updatedDraft]

	themeStore.saveCustomThemes(nextThemes)
	draftTheme.value = updatedDraft
	selectedPreset.value = updatedDraft.baseTheme
}

function beginPaletteInteraction(key: keyof CustomTheme['palette']) {
	if (!draftTheme.value) return
	const token = String(key)
	if (interactionToken.value === token) return
	interactionToken.value = token
	interactionSnapshot.value = cloneDraftTheme(draftTheme.value)
}

function commitPaletteInteraction(key: keyof CustomTheme['palette']) {
	if (!draftTheme.value) return
	const token = String(key)
	if (interactionToken.value !== token || !interactionSnapshot.value) return

	const snapshot = interactionSnapshot.value
	interactionSnapshot.value = null
	interactionToken.value = null

	if (JSON.stringify(snapshot) === JSON.stringify(draftTheme.value)) return

	undoHistory.value = [...undoHistory.value.slice(-79), snapshot]
	redoHistory.value = []
}

function applyDraftEdit(
	producer: (theme: CustomTheme) => CustomTheme,
	options: { syncBaseTheme?: boolean; recordHistory?: boolean } = {},
) {
	if (!draftTheme.value) return
	const currentDraft = cloneDraftTheme(draftTheme.value)
	if (!currentDraft) return
	const nextDraft = producer(currentDraft)
	if (JSON.stringify(currentDraft) === JSON.stringify(nextDraft)) return

	if (options.recordHistory !== false) {
		undoHistory.value = [...undoHistory.value.slice(-79), currentDraft]
	}
	redoHistory.value = []
	persistDraft(nextDraft, { syncBaseTheme: options.syncBaseTheme })
}

function updateField<K extends keyof CustomTheme>(key: K, value: CustomTheme[K]) {
	applyDraftEdit(
		(theme) => ({
			...theme,
			[key]: value,
		}),
		{ syncBaseTheme: key === 'baseTheme' },
	)
}

function updatePalette(key: keyof CustomTheme['palette'], value: string | number) {
	if (key === 'brandTintStrength' && typeof value === 'number') {
		const tintStrength = Math.max(0, Math.min(100, Math.round(value)))
		applyDraftEdit(
			(theme) => ({
				...theme,
				palette: {
					...theme.palette,
					brandTintStrength: tintStrength,
					brandHighlightOpacity: tintStrength,
					brandShadowOpacity: Math.max(14, Math.min(100, tintStrength + 10)),
					brandGradientStartOpacity: Math.max(12, Math.min(100, tintStrength)),
					brandGradientEndOpacity: Math.max(6, Math.min(100, tintStrength - 6)),
					brandGradientStrongEndOpacity: Math.max(10, Math.min(100, tintStrength - 2)),
					brandGradientBorderOpacity: Math.max(12, Math.min(100, tintStrength)),
				},
			}),
			{
				recordHistory: false,
			},
		)
		return
	}

	if (key === 'brandHighlightOpacity' && typeof value === 'number') {
		const tintStrength = Math.max(0, Math.min(100, Math.round(value)))
		applyDraftEdit(
			(theme) => ({
				...theme,
				palette: {
					...theme.palette,
					brandHighlightOpacity: tintStrength,
					brandTintStrength: tintStrength,
				},
			}),
			{
				recordHistory: false,
			},
		)
		return
	}

	applyDraftEdit((theme) => ({
		...theme,
		palette: { ...theme.palette, [key]: value as never },
	}), {
		recordHistory: false,
	})
}

function updatePaletteHex(key: keyof CustomTheme['palette'], event: Event) {
	const input = event.target as HTMLInputElement
	const value = input.value.trim().toUpperCase()
	if (!draftTheme.value) return
	if (!/^#([0-9A-F]{6}|[0-9A-F]{3})$/.test(value)) {
		input.value = String(draftTheme.value.palette[key] ?? '')
		return
	}
	beginPaletteInteraction(key)
	updatePalette(key, value)
	commitPaletteInteraction(key)
}

function undoDraft() {
	if (!draftTheme.value || undoHistory.value.length === 0) return
	interactionSnapshot.value = null
	interactionToken.value = null
	const previous = undoHistory.value[undoHistory.value.length - 1]
	undoHistory.value = undoHistory.value.slice(0, -1)
	redoHistory.value = [...redoHistory.value, cloneDraftTheme(draftTheme.value)!]
	persistDraft(previous, { syncBaseTheme: previous.baseTheme !== draftTheme.value.baseTheme })
}

function redoDraft() {
	if (!draftTheme.value || redoHistory.value.length === 0) return
	interactionSnapshot.value = null
	interactionToken.value = null
	const next = redoHistory.value[redoHistory.value.length - 1]
	redoHistory.value = redoHistory.value.slice(0, -1)
	undoHistory.value = [...undoHistory.value, cloneDraftTheme(draftTheme.value)!]
	persistDraft(next, { syncBaseTheme: next.baseTheme !== draftTheme.value.baseTheme })
}

async function duplicateDraft() {
	if (!draftTheme.value) return
	const duplicate = cloneCustomTheme(draftTheme.value)
	themeStore.saveCustomThemes([...themeStore.customThemes, duplicate])
	editorThemeId.value = duplicate.id
	setDraft(duplicate)
	await selectTheme(duplicate.id)
}

async function exportDraft() {
	if (!draftTheme.value) return
	const saved = await downloadCustomTheme(draftTheme.value)
	if (saved) notice.value = formatMessage(messages.exportTheme)
}

async function copyJson() {
	if (!draftTheme.value) return
	await navigator.clipboard.writeText(JSON.stringify(draftTheme.value, null, 2))
	notice.value = formatMessage(messages.copied)
}

async function deleteDraft() {
	if (!draftTheme.value) return
	const deletedId = draftTheme.value.id
	const themes = themeStore.customThemes.filter((theme) => theme.id !== deletedId)
	themeStore.saveCustomThemes(themes)
	if (themeStore.activeCustomThemeId === deletedId) {
		themeStore.clearCustomThemeSelection()
		await saveSettingsTheme(themeStore.selectedTheme)
	}
	editorThemeId.value = themes[0]?.id ?? null
	setDraft(themes[0] ?? null)
}

function openImport() {
	importInput.value?.click()
}

async function importTheme(event: Event) {
	const input = event.target as HTMLInputElement
	const file = input.files?.[0]
	input.value = ''
	if (!file) return
	try {
		const imported = normalizeImportedCustomTheme(JSON.parse(await file.text()))
		if (!imported) throw new Error('invalid-theme')
		if (themeStore.customThemes.some((theme) => theme.id === imported.id)) {
			imported.id = `${imported.id}-${Date.now().toString(36)}`
		}
		themeStore.saveCustomThemes([...themeStore.customThemes, imported])
		editorThemeId.value = imported.id
		setDraft(imported)
		await selectTheme(imported.id)
	} catch {
		importError.value = formatMessage(messages.importFailed)
	}
}
</script>

<template>
	<div class="studio">
		<header class="studio__intro studio__intro--compact">
			<div class="studio__intro-actions">
				<Button @click="createBlankTheme"><PlusIcon /> {{ formatMessage(messages.newTheme) }}</Button>
				<Button @click="openImport"><UploadIcon /> {{ formatMessage(messages.importTheme) }}</Button>
				<input ref="importInput" class="hidden" type="file" accept=".json,.revoria-theme.json" @change="importTheme" />
			</div>
		</header>

		<div v-if="importError" class="studio__notice studio__notice--error">{{ importError }}</div>
		<div v-else-if="notice" class="studio__notice">{{ notice }}</div>

		<div class="studio__layout">
			<aside class="studio__sidebar">
				<section class="studio-panel studio-panel--presets">
					<header class="studio-panel__header">
						<div>
							<div class="studio-panel__title">{{ formatMessage(messages.official) }}</div>
						</div>
					</header>

					<div class="preset-grid">
						<button
							v-for="theme in presetThemes"
							:key="theme"
							type="button"
							class="preset-tile"
							:class="{ 'preset-tile--active': selectedPreset === theme }"
							@click="pickPreset(theme)"
						>
							<span class="preset-tile__swatch" :class="getCustomThemePreviewClasses(theme)" />
							<span class="preset-tile__label">{{ themeLabel(theme) }}</span>
						</button>
					</div>

					<div class="preset-actions">
						<Button @click="createTheme"><PlusIcon /> {{ formatMessage(messages.createFromPreset) }}</Button>
					</div>
				</section>

				<section class="studio-panel studio-panel--library">
					<header class="studio-panel__header">
						<div class="studio-panel__title">{{ formatMessage(messages.custom) }}</div>
					</header>
					<div v-if="themeStore.customThemes.length === 0" class="studio__empty">{{ formatMessage(messages.empty) }}</div>
					<div v-else class="theme-library">
						<button
							v-for="theme in themeStore.customThemes"
							:key="theme.id"
							class="theme-card"
							:class="{ 'theme-card--active': editorThemeId === theme.id }"
							@click="selectTheme(theme.id)"
						>
							<div class="theme-card__swatch" :class="getCustomThemePreviewClasses(theme.baseTheme)" :style="customThemeToCssVariables(theme)" />
							<div class="theme-card__body">
								<div class="theme-card__title-row">
									<div class="theme-card__title">{{ theme.name }}</div>
									<span v-if="themeStore.activeCustomThemeId === theme.id" class="theme-card__pill">{{ formatMessage(messages.active) }}</span>
								</div>
								<div class="theme-card__meta">{{ themeLabel(theme.baseTheme) }}</div>
							</div>
						</button>
					</div>
				</section>
			</aside>

			<section class="studio-panel studio-panel--editor">
				<div v-if="draftTheme" class="editor">
					<header class="editor__header">
						<div>
							<div class="editor__title">{{ draftTheme.name }}</div>
							<div class="editor__sub">{{ formatMessage(messages.editor) }} · {{ themeLabel(draftTheme.baseTheme) }}</div>
						</div>
						<div class="editor__header-actions">
							<Button :disabled="undoHistory.length === 0" @click="undoDraft"><UndoIcon /> {{ formatMessage(messages.undo) }}</Button>
							<Button :disabled="redoHistory.length === 0" @click="redoDraft"><RedoIcon /> {{ formatMessage(messages.redo) }}</Button>
						</div>
					</header>

					<div class="editor-grid">
						<section class="editor-section">
							<div class="editor-section__title">{{ formatMessage(messages.metaSection) }}</div>
							<div class="editor__form">
								<label class="field">
									<span>{{ formatMessage(messages.name) }}</span>
									<input :value="draftTheme.name" type="text" @input="updateField('name', ($event.target as HTMLInputElement).value)" />
								</label>
								<label class="field">
									<span>{{ formatMessage(messages.author) }}</span>
									<input :value="draftTheme.author" type="text" @input="updateField('author', ($event.target as HTMLInputElement).value)" />
								</label>
								<label class="field field--full">
									<span>{{ formatMessage(messages.descriptionLabel) }}</span>
									<textarea :value="draftTheme.description" rows="3" @input="updateField('description', ($event.target as HTMLTextAreaElement).value)" />
								</label>
							</div>
						</section>

						<section class="editor-section">
							<div class="editor-section__title">{{ formatMessage(messages.colorsSection) }}</div>
							<div class="color-grid">
								<label v-for="field in coreFields" :key="field.key" class="color-field">
									<div class="field-heading"><span>{{ field.label }}</span></div>
									<div class="color-field__row">
										<input
											:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
											type="color"
											@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
											@input="updatePalette(field.key as keyof typeof draftTheme.palette, ($event.target as HTMLInputElement).value)"
											@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										/>
										<input :value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]" type="text" @change="updatePaletteHex(field.key as keyof typeof draftTheme.palette, $event)" />
									</div>
								</label>
							</div>
						</section>

						<section class="editor-section">
							<div class="editor-section__title">{{ formatMessage(messages.uiSection) }}</div>
							<div class="color-grid color-grid--compact">
								<label v-for="field in uiFields" :key="field.key" class="color-field">
									<div class="field-heading"><span>{{ field.label }}</span></div>
									<div class="color-field__row">
										<input
											:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
											type="color"
											@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
											@input="updatePalette(field.key as keyof typeof draftTheme.palette, ($event.target as HTMLInputElement).value)"
											@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										/>
										<input :value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]" type="text" @change="updatePaletteHex(field.key as keyof typeof draftTheme.palette, $event)" />
									</div>
								</label>
							</div>
							<div class="slider-grid">
								<label v-for="field in uiSliderFields" :key="field.key" class="slider-field">
									<div><span>{{ field.label }}</span><strong>{{ draftTheme.palette[field.key as keyof typeof draftTheme.palette] }}{{ field.suffix }}</strong></div>
									<input
										:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
										type="range"
										:min="field.min"
										:max="field.max"
										@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										@input="updatePalette(field.key as keyof typeof draftTheme.palette, Number(($event.target as HTMLInputElement).value))"
										@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
									/>
								</label>
							</div>
						</section>

						<section class="editor-section">
							<div class="editor-section__title">{{ formatMessage(messages.stateSection) }}</div>
							<div class="color-grid color-grid--compact">
								<label v-for="field in stateFields" :key="field.key" class="color-field">
									<div class="field-heading"><span>{{ field.label }}</span></div>
									<div class="color-field__row">
										<input
											:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
											type="color"
											@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
											@input="updatePalette(field.key as keyof typeof draftTheme.palette, ($event.target as HTMLInputElement).value)"
											@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										/>
										<input :value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]" type="text" @change="updatePaletteHex(field.key as keyof typeof draftTheme.palette, $event)" />
									</div>
								</label>
							</div>
						</section>

						<section class="editor-section">
							<div class="editor-section__title">{{ formatMessage(messages.accentSection) }}</div>
							<div class="color-grid color-grid--compact">
								<label v-for="field in accentColorFields" :key="field.key" class="color-field">
									<div class="field-heading"><span>{{ field.label }}</span></div>
									<div class="color-field__row">
										<input
											:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
											type="color"
											@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
											@input="updatePalette(field.key as keyof typeof draftTheme.palette, ($event.target as HTMLInputElement).value)"
											@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										/>
										<input :value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]" type="text" @change="updatePaletteHex(field.key as keyof typeof draftTheme.palette, $event)" />
									</div>
								</label>
							</div>
							<div class="slider-grid">
								<label v-for="field in accentSliderFields" :key="field.key" class="slider-field">
									<div><span>{{ field.label }}</span><strong>{{ draftTheme.palette[field.key as keyof typeof draftTheme.palette] }}{{ field.suffix }}</strong></div>
									<input
										:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
										type="range"
										:min="field.min"
										:max="field.max"
										@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										@input="updatePalette(field.key as keyof typeof draftTheme.palette, Number(($event.target as HTMLInputElement).value))"
										@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
									/>
								</label>
							</div>
						</section>

						<section class="editor-section">
							<div class="editor-section__title">{{ formatMessage(messages.glassSection) }}</div>
							<div class="editor__form editor__form--glass">
								<label v-for="field in glassColorFields" :key="field.key" class="color-field">
									<span>{{ field.label }}</span>
									<div class="color-field__row">
										<input
											:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
											type="color"
											@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
											@input="updatePalette(field.key as keyof typeof draftTheme.palette, ($event.target as HTMLInputElement).value)"
											@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										/>
										<input :value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]" type="text" @change="updatePaletteHex(field.key as keyof typeof draftTheme.palette, $event)" />
									</div>
								</label>
								<label v-for="field in glassSliderFields" :key="field.key" class="slider-field">
									<div><span>{{ field.label }}</span><strong>{{ draftTheme.palette[field.key as keyof typeof draftTheme.palette] }}{{ field.suffix }}</strong></div>
									<input
										:value="draftTheme.palette[field.key as keyof typeof draftTheme.palette]"
										type="range"
										:min="field.min"
										:max="field.max"
										@focus="beginPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
										@input="updatePalette(field.key as keyof typeof draftTheme.palette, Number(($event.target as HTMLInputElement).value))"
										@change="commitPaletteInteraction(field.key as keyof typeof draftTheme.palette)"
									/>
								</label>
							</div>
						</section>
					</div>

					<footer class="editor__toolbar">
						<Button @click="duplicateDraft"><CopyIcon /> {{ formatMessage(messages.duplicate) }}</Button>
						<Button @click="exportDraft"><DownloadIcon /> {{ formatMessage(messages.exportTheme) }}</Button>
						<Button @click="copyJson"><CopyIcon /> {{ formatMessage(messages.copyJson) }}</Button>
						<Button class="editor__delete" @click="deleteDraft"><TrashIcon /> {{ formatMessage(messages.deleteTheme) }}</Button>
					</footer>
				</div>

				<div v-else class="editor editor--empty">
					<div class="editor__title">{{ formatMessage(messages.custom) }}</div>
					<div class="editor__sub">{{ formatMessage(messages.empty) }}</div>
					<Button @click="createTheme"><PlusIcon /> {{ formatMessage(messages.createFromPreset) }}</Button>
				</div>
			</section>
		</div>
	</div>
</template>

<style scoped lang="scss">
.studio {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.studio__intro,
.editor__header,
.editor__toolbar {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
	flex-wrap: wrap;
}

.studio__intro--compact {
	justify-content: flex-end;
}

.studio__title,
.editor__title {
	margin: 0;
	font-size: 1.35rem;
	font-weight: 900;
	color: var(--color-contrast);
}

.studio__description,
.editor__sub,
.theme-card__meta,
.studio__notice,
.studio__empty {
	margin: 0;
	color: var(--color-secondary);
	line-height: 1.5;
}

.studio__intro-actions,
.editor__header-actions,
.preset-actions {
	display: flex;
	gap: 0.75rem;
	flex-wrap: wrap;
}

.studio__notice,
.studio__empty,
.preset-actions {
	padding: 0.9rem 1rem;
	border-radius: 1rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 78%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 94%, transparent);
}

.studio__notice--error {
	border-color: color-mix(in srgb, var(--color-red) 50%, var(--glass-border) 50%);
}

.studio__layout {
	display: grid;
	grid-template-columns: minmax(320px, 365px) minmax(0, 1fr);
	gap: 1rem;
	align-items: start;
}

.studio__sidebar {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.studio-panel {
	border-radius: 1.45rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 96%, transparent);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur));
}

.studio-panel--presets,
.studio-panel--library,
.studio-panel--editor {
	padding: 1.1rem;
}

.studio-panel__header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
	margin-bottom: 0.95rem;
}

.studio-panel__title,
.editor-section__title {
	font-size: 0.78rem;
	font-weight: 800;
	letter-spacing: 0.08em;
	text-transform: uppercase;
	color: var(--color-secondary);
}

.preset-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.7rem;
}

.preset-tile,
.theme-card {
	width: 100%;
	border: 1px solid color-mix(in srgb, var(--glass-border) 76%, transparent);
	background: color-mix(in srgb, var(--color-button-bg) 90%, transparent);
	color: var(--color-contrast);
	cursor: pointer;
	text-align: left;
	transition:
		transform 160ms ease,
		border-color 160ms ease,
		background 160ms ease,
		box-shadow 160ms ease;
}

.preset-tile:hover,
.theme-card:hover {
	transform: translateY(-1px);
	background: color-mix(in srgb, var(--color-button-bg-hover) 92%, transparent);
	box-shadow: 0 12px 24px color-mix(in srgb, black 12%, transparent);
}

.preset-tile {
	display: grid;
	grid-template-columns: auto minmax(0, 1fr);
	align-items: center;
	gap: 0.85rem;
	padding: 0.9rem;
	border-radius: 1.1rem;
	min-width: 0;
}

.preset-tile > * {
	min-width: 0;
}

.preset-tile--active,
.theme-card--active {
	border-color: color-mix(in srgb, var(--color-brand) 42%, var(--glass-border));
	background: color-mix(in srgb, var(--color-button-bg-selected) 28%, var(--color-button-bg) 72%);
	box-shadow:
		inset 0 0 0 1px color-mix(in srgb, var(--color-brand) 20%, transparent),
		0 16px 32px color-mix(in srgb, var(--color-brand-shadow) 16%, transparent);
}

.preset-tile__swatch,
.theme-card__swatch {
	flex-shrink: 0;
	border: 0;
}

.preset-tile__swatch {
	width: 2rem;
	height: 2rem;
	border-radius: 999px;
	background:
		linear-gradient(
			135deg,
			color-mix(in srgb, var(--surface-3) 88%, black 12%) 0%,
			color-mix(in srgb, var(--surface-4) 72%, var(--color-brand) 28%) 48%,
			var(--color-brand) 100%
		);
	box-shadow: none;
}

.preset-tile__label {
	display: -webkit-box;
	font-weight: 700;
	line-height: 1.25;
	min-width: 0;
	max-width: 100%;
	overflow: hidden;
	-webkit-line-clamp: 2;
	-webkit-box-orient: vertical;
	white-space: normal;
	overflow-wrap: break-word;
	word-break: normal;
	text-wrap: pretty;
	hyphens: manual;
}

.preset-actions {
	margin-top: 0.9rem;
}

.theme-library {
	display: flex;
	flex-direction: column;
	gap: 0.6rem;
}

.theme-card {
	display: grid;
	grid-template-columns: auto minmax(0, 1fr);
	gap: 0.8rem;
	align-items: center;
	padding: 0.8rem;
	border-radius: 1.1rem;
}

.theme-card__swatch {
	width: 3.1rem;
	height: 3rem;
	border-radius: 1rem;
	background:
		linear-gradient(
			135deg,
			color-mix(in srgb, var(--surface-3) 90%, black 10%) 0%,
			color-mix(in srgb, var(--surface-4) 72%, var(--color-brand) 28%) 48%,
			var(--color-brand) 100%
		);
	box-shadow: none;
}

.theme-card__body {
	min-width: 0;
}

.theme-card__title-row {
	display: flex;
	align-items: center;
	gap: 0.45rem;
	justify-content: space-between;
}

.theme-card__title {
	font-weight: 800;
	line-height: 1.2;
}

.theme-card__pill {
	flex-shrink: 0;
	padding: 0.15rem 0.45rem;
	border-radius: 999px;
	background: var(--color-button-bg-selected);
	color: var(--color-button-text-selected);
	font-size: 0.74rem;
	font-weight: 800;
}

.editor {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.editor-grid {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.editor-section {
	display: flex;
	flex-direction: column;
	gap: 0.85rem;
	padding: 1.05rem;
	border-radius: 1.15rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 70%, transparent);
	background: color-mix(in srgb, var(--color-button-bg) 74%, transparent);
}

.editor__form,
.color-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.85rem;
}

.slider-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.85rem;
}

.editor__form--glass {
	align-items: start;
}

.field,
.color-field,
.slider-field {
	display: flex;
	flex-direction: column;
	gap: 0.45rem;
}

.field span,
.color-field span,
.slider-field span {
	font-size: 0.86rem;
	font-weight: 700;
	color: var(--color-secondary);
}

.field-heading {
	display: flex;
	flex-direction: column;
	gap: 0;
}

.field input,
.field textarea,
.field select,
.color-field input[type='text'] {
	width: 100%;
	padding: 0.8rem 0.9rem;
	border-radius: 0.95rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 76%, transparent);
	background: color-mix(in srgb, var(--color-raised-bg) 92%, transparent);
	color: var(--color-contrast);
	outline: none;
	transition: border-color 160ms ease, box-shadow 160ms ease, background 160ms ease;
}

.field input:focus,
.field textarea:focus,
.field select:focus,
.color-field input[type='text']:focus {
	border-color: color-mix(in srgb, var(--color-brand) 42%, var(--color-button-border) 58%);
	box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-brand-highlight) 68%, transparent);
}

.field--full {
	grid-column: 1 / -1;
}

.color-field__row {
	display: grid;
	grid-template-columns: 3.2rem minmax(0, 1fr);
	gap: 0.6rem;
	align-items: center;
}

.color-field input[type='color'] {
	width: 3.2rem;
	height: 3rem;
	padding: 0;
	appearance: none;
	-webkit-appearance: none;
	border-radius: 0.95rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 76%, transparent);
	background: transparent;
	overflow: hidden;
	cursor: pointer;
}

.color-field input[type='color']::-webkit-color-swatch-wrapper {
	padding: 0;
}

.color-field input[type='color']::-webkit-color-swatch {
	border: 0;
	border-radius: 0.88rem;
}

.color-field input[type='color']::-moz-color-swatch {
	border: 0;
	border-radius: 0.88rem;
}

.slider-field div {
	display: flex;
	justify-content: space-between;
	gap: 0.75rem;
}

.slider-field input[type='range'] {
	width: 100%;
	accent-color: var(--color-brand);
}

.editor__delete {
	color: var(--color-red);
}

.editor--empty {
	justify-content: center;
	min-height: 24rem;
}

@media (max-width: 1200px) {
	.studio__layout {
		grid-template-columns: 1fr;
	}
}

@media (max-width: 780px) {
	.preset-grid,
	.editor__form,
	.color-grid,
	.slider-grid {
		grid-template-columns: 1fr;
	}
}
</style>
