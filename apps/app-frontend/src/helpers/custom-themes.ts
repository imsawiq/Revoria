import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { CSSProperties } from 'vue'

import type { ColorTheme } from '@/store/theme'

const CUSTOM_THEMES_STORAGE_KEY = 'revoria.customThemes'
const ACTIVE_CUSTOM_THEME_STORAGE_KEY = 'revoria.activeCustomThemeId'
const CUSTOM_THEME_FILE_TYPE = 'revoria-theme'
const CUSTOM_THEME_SCHEMA_VERSION = 4

const LIGHT_BASE_THEMES = ['light', 'rose-gold', 'cherry-blossom'] as const
const CUSTOM_THEME_BASE_OPTIONS = [
	'dark',
	'light',
	'oled',
	'retro',
	'sapphire',
	'amethyst',
	'sunset',
	'aurora',
	'nord',
	'cherry-cola',
	'slate',
	'rose-gold',
	'obsidian-gold',
	'cherry-blossom',
] as const satisfies readonly Exclude<ColorTheme, 'system'>[]

export type CustomThemeBase = (typeof CUSTOM_THEME_BASE_OPTIONS)[number]

export type CustomThemePalette = {
	surface1: string
	surface2: string
	surface3: string
	surface4: string
	surface5: string
	raisedBg: string
	raisedBgHover: string
	buttonBg: string
	buttonBgOpacity: number
	buttonBgHover: string
	buttonBgActive: string
	buttonBorder: string
	buttonBorderOpacity: number
	buttonSelected: string
	buttonSelectedOpacity: number
	buttonSelectedText: string
	divider: string
	textPrimary: string
	textDefault: string
	textSecondary: string
	brand: string
	brandHighlight: string
	brandHighlightOpacity: number
	brandShadow: string
	brandShadowOpacity: number
	brandGradientStart: string
	brandGradientStartOpacity: number
	brandGradientEnd: string
	brandGradientEndOpacity: number
	brandGradientStrongStart: string
	brandGradientStrongStartOpacity: number
	brandGradientStrongEnd: string
	brandGradientStrongEndOpacity: number
	brandGradientBorderColor: string
	brandGradientBorderOpacity: number
	loadingBarEnd: string
	success: string
	warning: string
	danger: string
	info: string
	utility: string
	brandTintStrength: number
	glassTint: string
	glassBorderColor: string
	glassOpacity: number
	glassStrongOpacity: number
	glassBorderOpacity: number
}

export type CustomTheme = {
	type: typeof CUSTOM_THEME_FILE_TYPE
	schemaVersion: typeof CUSTOM_THEME_SCHEMA_VERSION
	id: string
	name: string
	author: string
	description: string
	baseTheme: CustomThemeBase
	palette: CustomThemePalette
	createdAt: string
	updatedAt: string
}

const CUSTOM_THEME_DEFAULTS: CustomThemePalette = {
	surface1: '#0b0d10',
	surface2: '#101317',
	surface3: '#151a1f',
	surface4: '#1b2128',
	surface5: '#242c35',
	raisedBg: '#151a1f',
	raisedBgHover: '#202731',
	buttonBg: '#1b2128',
	buttonBgOpacity: 100,
	buttonBgHover: '#242c35',
	buttonBgActive: '#2b3440',
	buttonBorder: '#2f3946',
	buttonBorderOpacity: 9,
	buttonSelected: '#1BD96A',
	buttonSelectedOpacity: 16,
	buttonSelectedText: '#08110F',
	divider: '#2f3946',
	textPrimary: '#f3f7fa',
	textDefault: '#b8c1cc',
	textSecondary: '#96a1ad',
	brand: '#1bd96a',
	brandHighlight: '#1BD96A',
	brandHighlightOpacity: 18,
	brandShadow: '#1BD96A',
	brandShadowOpacity: 28,
	brandGradientStart: '#1BD96A',
	brandGradientStartOpacity: 18,
	brandGradientEnd: '#1BD96A',
	brandGradientEndOpacity: 8,
	brandGradientStrongStart: '#0B0D10',
	brandGradientStrongStartOpacity: 74,
	brandGradientStrongEnd: '#1BD96A',
	brandGradientStrongEndOpacity: 10,
	brandGradientBorderColor: '#1BD96A',
	brandGradientBorderOpacity: 12,
	loadingBarEnd: '#70FFAE',
	success: '#1BD96A',
	warning: '#FFA347',
	danger: '#FF496E',
	info: '#4A9EFF',
	utility: '#BC3FBC',
	brandTintStrength: 18,
	glassTint: '#0b0d10',
	glassBorderColor: '#FFFFFF',
	glassOpacity: 70,
	glassStrongOpacity: 86,
	glassBorderOpacity: 8,
}

function clamp(value: number, min: number, max: number) {
	return Math.max(min, Math.min(max, Math.round(value)))
}

function normalizeHex(value: string, fallback: string) {
	const hex = typeof value === 'string' ? value.trim() : ''
	return /^#([0-9a-f]{6}|[0-9a-f]{3})$/i.test(hex) ? hex.toUpperCase() : fallback
}

function componentToHex(value: number) {
	return Math.max(0, Math.min(255, Math.round(value))).toString(16).padStart(2, '0').toUpperCase()
}

function hexToRgb(value: string) {
	const hex = normalizeHex(value, '#000000').replace('#', '')
	const normalized =
		hex.length === 3
			? hex
					.split('')
					.map((chunk) => `${chunk}${chunk}`)
					.join('')
			: hex

	return {
		r: Number.parseInt(normalized.slice(0, 2), 16),
		g: Number.parseInt(normalized.slice(2, 4), 16),
		b: Number.parseInt(normalized.slice(4, 6), 16),
	}
}

function rgbToHex(r: number, g: number, b: number) {
	return `#${componentToHex(r)}${componentToHex(g)}${componentToHex(b)}`
}

function rgba(value: string, alphaPercent: number) {
	const { r, g, b } = hexToRgb(value)
	return `rgba(${r}, ${g}, ${b}, ${clamp(alphaPercent, 0, 100) / 100})`
}

function parseCssColor(value: string) {
	const input = value.trim()
	if (!input) return null

	if (input.startsWith('#')) {
		const { r, g, b } = hexToRgb(input)
		return { hex: rgbToHex(r, g, b), alpha: 1 }
	}

	const match = input.match(/^rgba?\((\d+)[,\s]+(\d+)[,\s]+(\d+)(?:[,\s/]+([0-9.]+))?\)$/i)
	if (!match) return null

	return {
		hex: rgbToHex(Number(match[1]), Number(match[2]), Number(match[3])),
		alpha: match[4] == null ? 1 : Number(match[4]),
	}
}

function getReadableContrast(hex: string) {
	const { r, g, b } = hexToRgb(hex)
	const luma = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255
	return luma > 0.62 ? '#08110F' : '#F8FFFB'
}

function shiftHex(hex: string, amount: number) {
	const { r, g, b } = hexToRgb(hex)
	return rgbToHex(r + amount, g + amount, b + amount)
}

function deriveLoadingBarEnd(brand: string) {
	return shiftHex(brand, 58)
}

function isLightCustomThemeBase(baseTheme: CustomThemeBase) {
	return LIGHT_BASE_THEMES.includes(baseTheme as (typeof LIGHT_BASE_THEMES)[number])
}

export function getCustomThemeBaseOptions() {
	return [...CUSTOM_THEME_BASE_OPTIONS]
}

export function getCustomThemePreviewClasses(baseTheme: CustomThemeBase) {
	const resolvedBaseTheme = isLightCustomThemeBase(baseTheme) ? 'light' : 'dark'
	const classes = [`${resolvedBaseTheme}-mode`]

	if (baseTheme !== resolvedBaseTheme) {
		classes.push(`${baseTheme}-mode`)
	}

	return classes
}

export function customThemeToCssVariables(theme: CustomTheme): CSSProperties {
	const palette = theme.palette
	const basePalette = snapshotPaletteFromTheme(theme.baseTheme)
	const buttonBorder = palette.buttonBorder
	const isLightBase = isLightCustomThemeBase(theme.baseTheme)
	const linkHover = getReadableContrast(palette.brand) === '#08110F'
		? rgbToHex(
				Math.max(0, hexToRgb(palette.brand).r - 18),
				Math.max(0, hexToRgb(palette.brand).g - 18),
				Math.max(0, hexToRgb(palette.brand).b - 18),
			)
		: rgbToHex(
				Math.min(255, hexToRgb(palette.brand).r + 18),
				Math.min(255, hexToRgb(palette.brand).g + 18),
				Math.min(255, hexToRgb(palette.brand).b + 18),
			)
	const buttonBgValue = rgba(palette.buttonBg, palette.buttonBgOpacity)
	const buttonBgHoverValue = palette.buttonBgHover
	const buttonBgActiveValue = palette.buttonBgActive
	const buttonBorderValue = rgba(buttonBorder, palette.buttonBorderOpacity)
	const selectedBgValue = rgba(palette.buttonSelected, palette.buttonSelectedOpacity)
	const neutralGradientButtonValue = `linear-gradient(180deg, ${isLightBase ? rgba(palette.buttonBg, 96) : palette.buttonBg} 0%, ${isLightBase ? rgba(palette.buttonBgActive, 96) : palette.buttonBgActive} 100%)`
	const brandGradientButtonValue = isLightBase ? 'rgba(255, 255, 255, 0.72)' : 'rgba(255, 255, 255, 0.08)'
	const brandGradientFadeOutValue = `linear-gradient(to bottom, ${rgba(palette.surface2, 0)} 0%, ${palette.surface2} 80%)`
	const brandHighlightValue = rgba(palette.brandHighlight, palette.brandHighlightOpacity)
	const brandShadowValue = rgba(palette.brandShadow, palette.brandShadowOpacity)
	const brandGradientBgValue = `linear-gradient(0deg, ${rgba(palette.brandGradientStart, palette.brandGradientStartOpacity)} 0%, ${rgba(palette.brandGradientEnd, palette.brandGradientEndOpacity)} 100%)`
	const brandGradientStrongBgValue = `linear-gradient(270deg, ${rgba(palette.brandGradientStrongStart, palette.brandGradientStrongStartOpacity)} 10%, ${rgba(palette.brandGradientStrongEnd, palette.brandGradientStrongEndOpacity)} 100%)`
	const brandGradientBorderValue = rgba(
		palette.brandGradientBorderColor,
		palette.brandGradientBorderOpacity,
	)
	const loadingBarGradientValue = `linear-gradient(to right, ${palette.brand} 0%, ${palette.loadingBarEnd} 100%)`
	const adHighlight = palette.utility
	const adBg = rgba(palette.utility, isLightBase ? 22 : 20)
	const adRaised = rgba(palette.utility, isLightBase ? 48 : 50)
	const adContrast = getReadableContrast(palette.utility)
	const differs = <K extends keyof CustomThemePalette>(key: K) => {
		const current = palette[key]
		const base = basePalette[key]
		if (typeof current === 'number' && typeof base === 'number') {
			return current !== base
		}
		return String(current).toUpperCase() !== String(base).toUpperCase()
	}
	const anyDiffers = (...keys: (keyof CustomThemePalette)[]) => keys.some((key) => differs(key))
	const variables: CSSProperties = {}
	const assign = (condition: boolean, entries: Record<string, string>) => {
		if (!condition) return
		Object.assign(variables, entries)
	}

	const surfaceChanged = anyDiffers('surface1', 'surface2', 'surface3', 'surface4', 'surface5')
	const textChanged = anyDiffers('textPrimary', 'textDefault', 'textSecondary')
	const brandChanged = anyDiffers('brand')
	const brandAtmosphereChanged = anyDiffers(
		'brandHighlight',
		'brandHighlightOpacity',
		'brandShadow',
		'brandShadowOpacity',
		'brandGradientStart',
		'brandGradientStartOpacity',
		'brandGradientEnd',
		'brandGradientEndOpacity',
		'brandGradientStrongStart',
		'brandGradientStrongStartOpacity',
		'brandGradientStrongEnd',
		'brandGradientStrongEndOpacity',
		'brandGradientBorderColor',
		'brandGradientBorderOpacity',
		'loadingBarEnd',
	)
	const glassChanged = anyDiffers(
		'glassTint',
		'glassBorderColor',
		'glassOpacity',
		'glassStrongOpacity',
		'glassBorderOpacity',
	)
	const raisedChanged = anyDiffers('raisedBg', 'raisedBgHover')
	const buttonChanged = anyDiffers(
		'buttonBg',
		'buttonBgOpacity',
		'buttonBgHover',
		'buttonBgActive',
		'buttonBorder',
		'buttonBorderOpacity',
		'buttonSelected',
		'buttonSelectedOpacity',
		'buttonSelectedText',
		'divider',
	)
	const semanticChanged = anyDiffers('success', 'warning', 'danger', 'info', 'utility')

	assign(surfaceChanged, {
		'--surface-1': palette.surface1,
		'--surface-2': palette.surface2,
		'--surface-3': palette.surface3,
		'--surface-4': palette.surface4,
		'--surface-5': palette.surface5,
		'--color-bg': palette.surface1,
		'--color-super-raised-bg': palette.surface4,
	})

	assign(textChanged, {
		'--color-text-primary': palette.textPrimary,
		'--color-text-default': palette.textDefault,
		'--color-text-tertiary': palette.textSecondary,
		'--color-base': palette.textDefault,
		'--color-secondary': palette.textSecondary,
		'--color-contrast': palette.textPrimary,
		'--color-button-text': palette.textDefault,
		'--color-button-text-hover': palette.textPrimary,
		'--color-button-text-active': palette.textPrimary,
		'--color-heading': palette.textPrimary,
		'--color-text': palette.textDefault,
		'--color-text-dark': palette.textPrimary,
		'--color-gray': palette.textSecondary,
		'--color-gray-highlight': rgba(palette.textSecondary, 25),
		'--color-scrollbar': rgba(palette.textSecondary, 46),
	})

	assign(brandChanged, {
		'--color-brand': palette.brand,
		'--color-primary': palette.brand,
		'--color-link': palette.brand,
		'--color-link-hover': linkHover,
		'--color-link-active': linkHover,
	})

	assign(brandChanged || brandAtmosphereChanged, {
		'--color-brand-highlight': brandHighlightValue,
		'--color-brand-shadow': brandShadowValue,
		'--brand-gradient-bg': brandGradientBgValue,
		'--brand-gradient-strong-bg': brandGradientStrongBgValue,
		'--brand-gradient-border': brandGradientBorderValue,
		'--brand-gradient-button': brandGradientButtonValue,
		'--brand-gradient-fade-out-color': brandGradientFadeOutValue,
		'--loading-bar-gradient': loadingBarGradientValue,
		'--color-banner-bg': brandHighlightValue,
		'--color-banner-side': palette.brand,
	})

	assign(glassChanged, {
		'--glass-bg-base': rgba(palette.glassTint, palette.glassOpacity),
		'--glass-bg-strong-base': rgba(palette.glassTint, palette.glassStrongOpacity),
		'--glass-border-base': rgba(palette.glassBorderColor, palette.glassBorderOpacity),
		'--glass-bg': rgba(palette.glassTint, palette.glassOpacity),
		'--glass-bg-strong': rgba(palette.glassTint, palette.glassStrongOpacity),
		'--glass-border': rgba(palette.glassBorderColor, palette.glassBorderOpacity),
		'--color-glass-bg': rgba(palette.glassTint, palette.glassOpacity),
		'--color-glass-bg-strong': rgba(palette.glassTint, palette.glassStrongOpacity),
	})

	assign(raisedChanged, {
		'--color-raised-bg': palette.raisedBg,
		'--color-raised-bg-hover': palette.raisedBgHover,
	})

	assign(buttonChanged, {
		'--color-button-bg': buttonBgValue,
		'--color-button-bg-hover': buttonBgHoverValue,
		'--color-button-bg-active': buttonBgActiveValue,
		'--color-button-border': buttonBorderValue,
		'--color-divider': palette.divider,
		'--color-divider-dark': palette.divider,
		'--color-button-bg-selected': selectedBgValue,
		'--color-button-text-selected': palette.buttonSelectedText,
		'--color-accent-contrast': palette.buttonSelectedText,
		'--color-selected-button-bg': selectedBgValue,
		'--color-gradient-button-bg': neutralGradientButtonValue,
		'--timeline-line-color': palette.divider,
	})

	assign(semanticChanged, {
		'--color-success': palette.success,
		'--color-danger': palette.danger,
		'--color-special-orange': palette.warning,
		'--color-ad': adBg,
		'--color-ad-raised': adRaised,
		'--color-ad-contrast': adContrast,
		'--color-ad-highlight': adHighlight,
		'--medal-promotion-bg': rgba(palette.surface2, 92),
		'--medal-promotion-bg-gradient': `linear-gradient(135deg, ${rgba(palette.surface1, 82)} 0%, ${rgba(palette.warning, 18)} 100%)`,
		'--medal-promotion-bg-orange': rgba(palette.warning, 24),
		'--medal-promotion-text-orange': palette.warning,
		'--color-green': palette.success,
		'--color-green-highlight': rgba(palette.success, 18),
		'--color-green-bg': rgba(palette.success, 18),
		'--color-orange': palette.warning,
		'--color-orange-highlight': rgba(palette.warning, 18),
		'--color-orange-bg': rgba(palette.warning, 18),
		'--color-red': palette.danger,
		'--color-red-highlight': rgba(palette.danger, 18),
		'--color-red-bg': rgba(palette.danger, 18),
		'--color-blue': palette.info,
		'--color-blue-highlight': rgba(palette.info, 18),
		'--color-blue-bg': rgba(palette.info, 18),
		'--color-purple': palette.utility,
		'--color-purple-highlight': rgba(palette.utility, 18),
		'--color-purple-bg': rgba(palette.utility, 18),
	})

	assign(textChanged || brandChanged, {
		'--color-banner-text': palette.textPrimary,
	})

	return variables
}

const RESET_VARIABLES = [
	'--surface-1',
	'--surface-2',
	'--surface-3',
	'--surface-4',
	'--surface-5',
	'--color-text-primary',
	'--color-text-default',
	'--color-text-tertiary',
	'--color-brand',
	'--color-brand-highlight',
	'--color-brand-shadow',
	'--glass-bg-base',
	'--glass-bg-strong-base',
	'--glass-border-base',
	'--glass-bg',
	'--glass-bg-strong',
	'--glass-border',
	'--color-glass-bg',
	'--color-glass-bg-strong',
	'--color-bg',
	'--color-raised-bg',
	'--color-super-raised-bg',
	'--color-base',
	'--color-secondary',
	'--color-contrast',
	'--color-button-bg',
	'--color-button-bg-hover',
	'--color-button-bg-active',
	'--color-button-border',
	'--color-button-text',
	'--color-button-text-hover',
	'--color-button-text-active',
	'--color-divider',
	'--color-divider-dark',
	'--color-raised-bg-hover',
	'--color-button-bg-selected',
	'--color-button-text-selected',
	'--color-accent-contrast',
	'--color-primary',
	'--color-success',
	'--color-danger',
	'--color-heading',
	'--color-text',
	'--color-text-dark',
	'--color-selected-button-bg',
	'--color-gradient-button-bg',
	'--color-gray',
	'--color-gray-highlight',
	'--color-special-orange',
	'--color-link',
	'--color-link-hover',
	'--color-link-active',
	'--color-scrollbar',
	'--brand-gradient-bg',
	'--brand-gradient-strong-bg',
	'--brand-gradient-border',
	'--brand-gradient-button',
	'--brand-gradient-fade-out-color',
	'--loading-bar-gradient',
	'--color-ad',
	'--color-ad-raised',
	'--color-ad-contrast',
	'--color-ad-highlight',
	'--color-banner-bg',
	'--color-banner-side',
	'--color-banner-text',
	'--medal-promotion-bg',
	'--medal-promotion-bg-gradient',
	'--medal-promotion-bg-orange',
	'--medal-promotion-text-orange',
	'--timeline-line-color',
	'--color-green',
	'--color-green-highlight',
	'--color-green-bg',
	'--color-orange',
	'--color-orange-highlight',
	'--color-orange-bg',
	'--color-red',
	'--color-red-highlight',
	'--color-red-bg',
	'--color-blue',
	'--color-blue-highlight',
	'--color-blue-bg',
	'--color-purple',
	'--color-purple-highlight',
	'--color-purple-bg',
]

export function clearAppliedCustomTheme() {
	if (typeof document === 'undefined') return
	const root = document.documentElement
	for (const variable of RESET_VARIABLES) {
		root.style.removeProperty(variable)
	}
}

export function applyCustomThemeToDocument(theme: CustomTheme | null | undefined) {
	if (typeof document === 'undefined') return
	clearAppliedCustomTheme()
	if (!theme) return

	const root = document.documentElement
	const variables = customThemeToCssVariables(theme)
	for (const [key, value] of Object.entries(variables)) {
		root.style.setProperty(key, String(value))
	}
}

function parseNumber(value: unknown, fallback: number, min = 0, max = 100) {
	if (typeof value !== 'number' || Number.isNaN(value)) return fallback
	return clamp(value, min, max)
}

function resolveVariableColor(
	container: HTMLElement,
	property: 'backgroundColor' | 'color' | 'borderTopColor',
	variable: string,
	fallback: string,
) {
	const sample = document.createElement('div')
	sample.style.position = 'absolute'
	sample.style.pointerEvents = 'none'
	sample.style.opacity = '0'

	if (property === 'backgroundColor') {
		sample.style.backgroundColor = `var(${variable})`
	} else if (property === 'color') {
		sample.style.color = `var(${variable})`
	} else {
		sample.style.borderTop = `1px solid var(${variable})`
	}

	container.appendChild(sample)
	const computed = getComputedStyle(sample)
	const raw =
		property === 'backgroundColor'
			? computed.backgroundColor
			: property === 'color'
				? computed.color
				: computed.borderTopColor
	sample.remove()
	return parseCssColor(raw) ?? parseCssColor(fallback)
}

function extractGradientStops(value: string) {
	return [...value.matchAll(/rgba?\([^)]+\)|#[0-9a-f]{3,8}/gi)]
		.map((match) => parseCssColor(match[0]))
		.filter((color): color is NonNullable<typeof color> => color !== null)
}

function resolveVariableGradient(container: HTMLElement, variable: string, fallback: string) {
	const sample = document.createElement('div')
	sample.style.position = 'absolute'
	sample.style.pointerEvents = 'none'
	sample.style.opacity = '0'
	sample.style.backgroundImage = `var(${variable})`
	container.appendChild(sample)
	const raw = getComputedStyle(sample).backgroundImage
	sample.remove()
	const resolved = extractGradientStops(raw)
	return resolved.length >= 2 ? resolved : extractGradientStops(fallback)
}

function repairLegacyPalette(
	input: Partial<CustomThemePalette> | undefined,
	baseTheme: CustomThemeBase,
	schemaVersion: number | undefined,
) {
	const palette = { ...(input ?? {}) }
	if ((schemaVersion ?? 0) >= CUSTOM_THEME_SCHEMA_VERSION) return palette

	const snapshot = snapshotPaletteFromTheme(baseTheme)
	const colorKeys = [
		'surface1',
		'surface2',
		'surface3',
		'surface4',
		'surface5',
		'raisedBg',
		'raisedBgHover',
		'buttonBg',
		'buttonBgHover',
		'buttonBgActive',
		'buttonBorder',
		'buttonSelected',
		'buttonSelectedText',
		'divider',
		'textPrimary',
		'textDefault',
		'textSecondary',
		'brand',
		'brandHighlight',
		'brandShadow',
		'brandGradientStart',
		'brandGradientEnd',
		'brandGradientStrongStart',
		'brandGradientStrongEnd',
		'brandGradientBorderColor',
		'loadingBarEnd',
		'success',
		'warning',
		'danger',
		'info',
		'utility',
		'glassTint',
	] as const satisfies readonly (keyof CustomThemePalette)[]

	for (const key of colorKeys) {
		const current = palette[key]
		const fallback = CUSTOM_THEME_DEFAULTS[key]
		const snapshotValue = snapshot[key]
		if (typeof current !== 'string') continue
		if (normalizeHex(current, fallback) === fallback && snapshotValue !== fallback) {
			palette[key] = snapshotValue
		}
	}

	const numberKeys = [
		'buttonBgOpacity',
		'buttonBorderOpacity',
		'buttonSelectedOpacity',
		'brandHighlightOpacity',
		'brandShadowOpacity',
		'brandGradientStartOpacity',
		'brandGradientEndOpacity',
		'brandGradientStrongStartOpacity',
		'brandGradientStrongEndOpacity',
		'brandGradientBorderOpacity',
		'brandTintStrength',
		'glassOpacity',
		'glassStrongOpacity',
		'glassBorderOpacity',
	] as const satisfies readonly (keyof CustomThemePalette)[]

	for (const key of numberKeys) {
		const current = palette[key]
		const fallback = CUSTOM_THEME_DEFAULTS[key]
		const snapshotValue = snapshot[key]
		if (typeof current !== 'number') continue
		if (current === fallback && snapshotValue !== fallback) {
			palette[key] = snapshotValue
		}
	}

	return palette
}

function normalizePalette(
	input: Partial<CustomThemePalette> | undefined,
	fallbackPalette?: Partial<CustomThemePalette>,
): CustomThemePalette {
	const palette = input ?? {}
	const fallback = { ...CUSTOM_THEME_DEFAULTS, ...(fallbackPalette ?? {}) }
	const surface1 = normalizeHex(palette.surface1 ?? '', fallback.surface1)
	const surface2 = normalizeHex(palette.surface2 ?? '', fallback.surface2)
	const surface3 = normalizeHex(palette.surface3 ?? '', fallback.surface3)
	const surface4 = normalizeHex(palette.surface4 ?? '', fallback.surface4)
	const surface5 = normalizeHex(palette.surface5 ?? '', fallback.surface5)
	const textPrimary = normalizeHex(palette.textPrimary ?? '', fallback.textPrimary)
	const textDefault = normalizeHex(palette.textDefault ?? '', fallback.textDefault)
	const textSecondary = normalizeHex(palette.textSecondary ?? '', fallback.textSecondary)
	const brand = normalizeHex(palette.brand ?? '', fallback.brand)
	const brandHighlight = normalizeHex(
		palette.brandHighlight ?? palette.brand ?? '',
		fallback.brandHighlight ?? brand,
	)
	const brandHighlightOpacity = parseNumber(
		palette.brandHighlightOpacity,
		fallback.brandHighlightOpacity ?? parseNumber(palette.brandTintStrength, 18, 0, 100),
		0,
		100,
	)
	const brandShadow = normalizeHex(
		palette.brandShadow ?? palette.brand ?? '',
		fallback.brandShadow ?? brand,
	)
	const brandShadowOpacity = parseNumber(
		palette.brandShadowOpacity,
		fallback.brandShadowOpacity ??
			clamp(
				parseNumber(palette.brandTintStrength, 18, 0, 100) + 10,
				0,
				100,
			),
		0,
		100,
	)
	const brandGradientStart = normalizeHex(
		palette.brandGradientStart ?? palette.brand ?? '',
		fallback.brandGradientStart ?? brand,
	)
	const brandGradientStartOpacity = parseNumber(
		palette.brandGradientStartOpacity,
		fallback.brandGradientStartOpacity ?? brandHighlightOpacity,
		0,
		100,
	)
	const brandGradientEnd = normalizeHex(
		palette.brandGradientEnd ?? palette.brand ?? '',
		fallback.brandGradientEnd ?? brand,
	)
	const brandGradientEndOpacity = parseNumber(
		palette.brandGradientEndOpacity,
		fallback.brandGradientEndOpacity ??
			clamp(Math.max(4, brandHighlightOpacity - 10), 0, 100),
		0,
		100,
	)
	const brandGradientStrongStart = normalizeHex(
		palette.brandGradientStrongStart ?? palette.surface1 ?? '',
		fallback.brandGradientStrongStart ?? surface1,
	)
	const brandGradientStrongStartOpacity = parseNumber(
		palette.brandGradientStrongStartOpacity,
		fallback.brandGradientStrongStartOpacity,
		0,
		100,
	)
	const brandGradientStrongEnd = normalizeHex(
		palette.brandGradientStrongEnd ?? palette.brand ?? '',
		fallback.brandGradientStrongEnd ?? brand,
	)
	const brandGradientStrongEndOpacity = parseNumber(
		palette.brandGradientStrongEndOpacity,
		fallback.brandGradientStrongEndOpacity ??
			clamp(Math.max(6, brandHighlightOpacity - 8), 0, 100),
		0,
		100,
	)
	const brandGradientBorderColor = normalizeHex(
		palette.brandGradientBorderColor ?? palette.brand ?? '',
		fallback.brandGradientBorderColor ?? brand,
	)
	const brandGradientBorderOpacity = parseNumber(
		palette.brandGradientBorderOpacity,
		fallback.brandGradientBorderOpacity ??
			clamp(Math.max(8, brandHighlightOpacity - 2), 0, 100),
		0,
		100,
	)
	const loadingBarEnd = normalizeHex(
		palette.loadingBarEnd ?? '',
		fallback.loadingBarEnd ?? deriveLoadingBarEnd(brand),
	)
	const divider = normalizeHex(palette.divider ?? '', fallback.divider)
	const raisedBg = normalizeHex(palette.raisedBg ?? '', fallback.raisedBg)
	const raisedBgHover = normalizeHex(
		palette.raisedBgHover ?? palette.surface5 ?? palette.raisedBg ?? '',
		fallback.raisedBgHover,
	)
	const buttonBg = normalizeHex(
		palette.buttonBg ?? palette.surface3 ?? palette.raisedBg ?? '',
		fallback.buttonBg,
	)
	const buttonBgOpacity = parseNumber(
		palette.buttonBgOpacity,
		fallback.buttonBgOpacity,
		0,
		100,
	)
	const buttonBgHover = normalizeHex(
		palette.buttonBgHover ?? palette.raisedBgHover ?? palette.buttonBg ?? '',
		fallback.buttonBgHover,
	)
	const buttonBgActive = normalizeHex(
		palette.buttonBgActive ?? palette.buttonBgHover ?? palette.buttonBg ?? '',
		fallback.buttonBgActive,
	)
	const buttonSelected = normalizeHex(palette.buttonSelected ?? '', fallback.buttonSelected)
	return {
		surface1,
		surface2,
		surface3,
		surface4,
		surface5,
		raisedBg,
		raisedBgHover,
		buttonBg,
		buttonBgOpacity,
		buttonBgHover,
		buttonBgActive,
		buttonBorder: normalizeHex(
			palette.buttonBorder ?? palette.divider ?? palette.surface4 ?? '',
			fallback.buttonBorder ?? divider,
		),
		buttonBorderOpacity: parseNumber(
			palette.buttonBorderOpacity,
			fallback.buttonBorderOpacity,
			0,
			100,
		),
		buttonSelected,
		buttonSelectedOpacity: parseNumber(
			palette.buttonSelectedOpacity,
			fallback.buttonSelectedOpacity,
			0,
			100,
		),
		buttonSelectedText: normalizeHex(
			palette.buttonSelectedText ?? '',
			fallback.buttonSelectedText ?? getReadableContrast(buttonSelected),
		),
		divider,
		textPrimary,
		textDefault,
		textSecondary,
		brand,
		brandHighlight,
		brandHighlightOpacity,
		brandShadow,
		brandShadowOpacity,
		brandGradientStart,
		brandGradientStartOpacity,
		brandGradientEnd,
		brandGradientEndOpacity,
		brandGradientStrongStart,
		brandGradientStrongStartOpacity,
		brandGradientStrongEnd,
		brandGradientStrongEndOpacity,
		brandGradientBorderColor,
		brandGradientBorderOpacity,
		loadingBarEnd,
		success: normalizeHex(palette.success ?? '', fallback.success),
		warning: normalizeHex(palette.warning ?? '', fallback.warning),
		danger: normalizeHex(palette.danger ?? '', fallback.danger),
		info: normalizeHex(palette.info ?? '', fallback.info),
		utility: normalizeHex(palette.utility ?? '', fallback.utility),
		brandTintStrength: parseNumber(
			palette.brandTintStrength,
			fallback.brandTintStrength,
			4,
			40,
		),
		glassTint: normalizeHex(palette.glassTint ?? palette.surface1 ?? '', fallback.glassTint ?? surface1),
		glassBorderColor: normalizeHex(
			palette.glassBorderColor ?? palette.buttonBorder ?? '',
			fallback.glassBorderColor,
		),
		glassOpacity: parseNumber(palette.glassOpacity, fallback.glassOpacity, 10, 100),
		glassStrongOpacity: parseNumber(
			palette.glassStrongOpacity,
			fallback.glassStrongOpacity,
			10,
			100,
		),
		glassBorderOpacity: parseNumber(
			palette.glassBorderOpacity,
			fallback.glassBorderOpacity,
			0,
			40,
		),
	}
}

function ensureBaseTheme(baseTheme: unknown): CustomThemeBase {
	return CUSTOM_THEME_BASE_OPTIONS.includes(baseTheme as CustomThemeBase)
		? (baseTheme as CustomThemeBase)
		: 'dark'
}

function safeParseStorage<T>(key: string, fallback: T): T {
	try {
		if (typeof window === 'undefined') return fallback
		const raw = window.localStorage.getItem(key)
		return raw ? (JSON.parse(raw) as T) : fallback
	} catch {
		return fallback
	}
}

function writeStorage(key: string, value: unknown) {
	try {
		if (typeof window === 'undefined') return
		if (value == null || value === '') {
			window.localStorage.removeItem(key)
			return
		}
		window.localStorage.setItem(key, JSON.stringify(value))
	} catch {
		// ignore storage failures
	}
}

export function loadStoredCustomThemes(): CustomTheme[] {
	const rawThemes = safeParseStorage<CustomTheme[]>(CUSTOM_THEMES_STORAGE_KEY, [])
	return rawThemes
		.map((theme) => normalizeImportedCustomTheme(theme))
		.filter((theme): theme is CustomTheme => theme !== null)
}

export function persistCustomThemes(themes: CustomTheme[]) {
	writeStorage(CUSTOM_THEMES_STORAGE_KEY, themes)
}

export function loadStoredActiveCustomThemeId(): string | null {
	return safeParseStorage<string | null>(ACTIVE_CUSTOM_THEME_STORAGE_KEY, null)
}

export function persistActiveCustomThemeId(id: string | null) {
	writeStorage(ACTIVE_CUSTOM_THEME_STORAGE_KEY, id)
}

function makeThemeId() {
	return `custom-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`
}

export function createEmptyCustomTheme(baseTheme: CustomThemeBase, palette: CustomThemePalette): CustomTheme {
	const now = new Date().toISOString()
	return {
		type: CUSTOM_THEME_FILE_TYPE,
		schemaVersion: CUSTOM_THEME_SCHEMA_VERSION,
		id: makeThemeId(),
		name: 'New custom theme',
		author: '',
		description: '',
		baseTheme,
		palette,
		createdAt: now,
		updatedAt: now,
	}
}

export function normalizeImportedCustomTheme(input: unknown): CustomTheme | null {
	if (!input || typeof input !== 'object') return null
	const source = input as Partial<CustomTheme>
	if (source.type !== CUSTOM_THEME_FILE_TYPE) return null

	const createdAt = typeof source.createdAt === 'string' ? source.createdAt : new Date().toISOString()
	const updatedAt = typeof source.updatedAt === 'string' ? source.updatedAt : createdAt
	const baseTheme = ensureBaseTheme(source.baseTheme)
	const fallbackPalette = snapshotPaletteFromTheme(baseTheme)
	const repairedPalette = repairLegacyPalette(
		source.palette,
		baseTheme,
		typeof source.schemaVersion === 'number' ? source.schemaVersion : undefined,
	)

	return {
		type: CUSTOM_THEME_FILE_TYPE,
		schemaVersion: CUSTOM_THEME_SCHEMA_VERSION,
		id: typeof source.id === 'string' && source.id.trim().length > 0 ? source.id : makeThemeId(),
		name: typeof source.name === 'string' && source.name.trim().length > 0 ? source.name.trim() : 'Imported theme',
		author: typeof source.author === 'string' ? source.author : '',
		description: typeof source.description === 'string' ? source.description : '',
		baseTheme,
		palette: normalizePalette(repairedPalette, fallbackPalette),
		createdAt,
		updatedAt,
	}
}

function readStyleValue(styles: CSSStyleDeclaration, variable: string, fallback: string) {
	return styles.getPropertyValue(variable)?.trim() || fallback
}

function readThemeHex(
	styles: CSSStyleDeclaration,
	variable: string,
	fallback: string,
	alternativeVariable?: string,
) {
	const primary = parseCssColor(readStyleValue(styles, variable, ''))
	if (primary?.hex) return normalizeHex(primary.hex, fallback)
	if (alternativeVariable) {
		const alternative = parseCssColor(readStyleValue(styles, alternativeVariable, ''))
		if (alternative?.hex) return normalizeHex(alternative.hex, fallback)
	}
	return normalizeHex(readStyleValue(styles, variable, fallback), fallback)
}

export function snapshotPaletteFromTheme(baseTheme: CustomThemeBase): CustomThemePalette {
	if (typeof document === 'undefined') {
		return { ...CUSTOM_THEME_DEFAULTS }
	}

	const probe = document.createElement('div')
	probe.className = getCustomThemePreviewClasses(baseTheme).join(' ')
	probe.style.position = 'fixed'
	probe.style.inset = '-9999px auto auto -9999px'
	probe.style.pointerEvents = 'none'
	document.body.appendChild(probe)

	const styles = getComputedStyle(probe)
	const brandHighlight = parseCssColor(readStyleValue(styles, '--color-brand-highlight', 'rgba(27, 217, 106, 0.18)'))
	const brandShadow = parseCssColor(readStyleValue(styles, '--color-brand-shadow', 'rgba(27, 217, 106, 0.28)'))
	const brandGradientBg = resolveVariableGradient(
		probe,
		'--brand-gradient-bg',
		'linear-gradient(0deg, rgba(27, 217, 106, 0.18) 0%, rgba(27, 217, 106, 0.08) 100%)',
	)
	const brandGradientStrongBg = resolveVariableGradient(
		probe,
		'--brand-gradient-strong-bg',
		'linear-gradient(270deg, rgba(11, 13, 16, 0.74) 10%, rgba(27, 217, 106, 0.10) 100%)',
	)
	const loadingBarGradient = resolveVariableGradient(
		probe,
		'--loading-bar-gradient',
		'linear-gradient(to right, #1BD96A 0%, #70FFAE 100%)',
	)
	const glassBg =
		resolveVariableColor(probe, 'backgroundColor', '--glass-bg-base', 'rgba(11, 13, 16, 0.7)') ??
		parseCssColor('rgba(11, 13, 16, 0.7)')
	const glassBgStrong =
		resolveVariableColor(
			probe,
			'backgroundColor',
			'--glass-bg-strong-base',
			'rgba(14, 17, 21, 0.86)',
		) ?? parseCssColor('rgba(14, 17, 21, 0.86)')
	const glassBorder =
		resolveVariableColor(probe, 'borderTopColor', '--glass-border-base', 'rgba(255,255,255,0.08)') ??
		parseCssColor('rgba(255,255,255,0.08)')
	const buttonBg =
		resolveVariableColor(probe, 'backgroundColor', '--color-button-bg', 'rgba(27, 33, 40, 1)') ??
		parseCssColor('rgba(27, 33, 40, 1)')
	const buttonBgHover =
		resolveVariableColor(probe, 'backgroundColor', '--color-button-bg-hover', 'rgba(36, 44, 53, 1)') ??
		parseCssColor('rgba(36, 44, 53, 1)')
	const buttonBgActive =
		resolveVariableColor(probe, 'backgroundColor', '--color-button-bg-active', 'rgba(43, 52, 64, 1)') ??
		parseCssColor('rgba(43, 52, 64, 1)')
	const buttonBorder =
		resolveVariableColor(probe, 'borderTopColor', '--color-button-border', 'rgba(47, 57, 70, 0.09)') ??
		parseCssColor('rgba(47, 57, 70, 0.09)')
	const buttonSelected =
		resolveVariableColor(probe, 'backgroundColor', '--color-button-bg-selected', 'rgba(27, 217, 106, 0.16)') ??
		parseCssColor('rgba(27, 217, 106, 0.16)')
	const brandGradientBorder =
		resolveVariableColor(probe, 'borderTopColor', '--brand-gradient-border', 'rgba(27, 217, 106, 0.12)') ??
		parseCssColor('rgba(27, 217, 106, 0.12)')
	const buttonSelectedText =
		resolveVariableColor(probe, 'color', '--color-button-text-selected', '#08110F') ??
		parseCssColor('#08110F')
	const raisedBgHover =
		resolveVariableColor(probe, 'backgroundColor', '--color-raised-bg-hover', '#202731') ??
		parseCssColor('#202731')

	const palette = {
		surface1: normalizeHex(
			resolveVariableColor(probe, 'backgroundColor', '--color-bg', CUSTOM_THEME_DEFAULTS.surface1)?.hex ??
				CUSTOM_THEME_DEFAULTS.surface1,
			CUSTOM_THEME_DEFAULTS.surface1,
		),
		surface2: readThemeHex(styles, '--surface-2', CUSTOM_THEME_DEFAULTS.surface2, '--color-raised-bg'),
		surface3: readThemeHex(styles, '--surface-3', CUSTOM_THEME_DEFAULTS.surface3, '--color-button-bg'),
		surface4: readThemeHex(styles, '--surface-4', CUSTOM_THEME_DEFAULTS.surface4, '--color-super-raised-bg'),
		surface5: readThemeHex(styles, '--surface-5', CUSTOM_THEME_DEFAULTS.surface5, '--color-raised-bg-hover'),
		raisedBg: readThemeHex(styles, '--color-raised-bg', CUSTOM_THEME_DEFAULTS.raisedBg),
		raisedBgHover: normalizeHex(raisedBgHover?.hex ?? CUSTOM_THEME_DEFAULTS.raisedBgHover, CUSTOM_THEME_DEFAULTS.raisedBgHover),
		buttonBg: normalizeHex(buttonBg?.hex ?? CUSTOM_THEME_DEFAULTS.buttonBg, CUSTOM_THEME_DEFAULTS.buttonBg),
		buttonBgOpacity: clamp(Math.round((buttonBg?.alpha ?? 1) * 100), 0, 100),
		buttonBgHover: normalizeHex(buttonBgHover?.hex ?? CUSTOM_THEME_DEFAULTS.buttonBgHover, CUSTOM_THEME_DEFAULTS.buttonBgHover),
		buttonBgActive: normalizeHex(buttonBgActive?.hex ?? CUSTOM_THEME_DEFAULTS.buttonBgActive, CUSTOM_THEME_DEFAULTS.buttonBgActive),
		buttonBorder: normalizeHex(buttonBorder?.hex ?? CUSTOM_THEME_DEFAULTS.buttonBorder, CUSTOM_THEME_DEFAULTS.buttonBorder),
		buttonBorderOpacity: clamp(Math.round((buttonBorder?.alpha ?? 0.09) * 100), 0, 100),
		buttonSelected: normalizeHex(buttonSelected?.hex ?? CUSTOM_THEME_DEFAULTS.buttonSelected, CUSTOM_THEME_DEFAULTS.buttonSelected),
		buttonSelectedOpacity: clamp(Math.round((buttonSelected?.alpha ?? 0.16) * 100), 0, 100),
		buttonSelectedText: normalizeHex(
			buttonSelectedText?.hex ?? CUSTOM_THEME_DEFAULTS.buttonSelectedText,
			CUSTOM_THEME_DEFAULTS.buttonSelectedText,
		),
		divider: readThemeHex(styles, '--color-divider', CUSTOM_THEME_DEFAULTS.divider, '--glass-border'),
		textPrimary: normalizeHex(
			resolveVariableColor(probe, 'color', '--color-contrast', CUSTOM_THEME_DEFAULTS.textPrimary)?.hex ??
				CUSTOM_THEME_DEFAULTS.textPrimary,
			CUSTOM_THEME_DEFAULTS.textPrimary,
		),
		textDefault: normalizeHex(
			resolveVariableColor(probe, 'color', '--color-base', CUSTOM_THEME_DEFAULTS.textDefault)?.hex ??
				CUSTOM_THEME_DEFAULTS.textDefault,
			CUSTOM_THEME_DEFAULTS.textDefault,
		),
		textSecondary: normalizeHex(
			resolveVariableColor(probe, 'color', '--color-secondary', CUSTOM_THEME_DEFAULTS.textSecondary)?.hex ??
				CUSTOM_THEME_DEFAULTS.textSecondary,
			CUSTOM_THEME_DEFAULTS.textSecondary,
		),
		brand: readThemeHex(styles, '--color-brand', CUSTOM_THEME_DEFAULTS.brand),
		brandHighlight: normalizeHex(
			brandHighlight?.hex ?? CUSTOM_THEME_DEFAULTS.brandHighlight,
			CUSTOM_THEME_DEFAULTS.brandHighlight,
		),
		brandHighlightOpacity: clamp(
			Math.round((brandHighlight?.alpha ?? CUSTOM_THEME_DEFAULTS.brandHighlightOpacity / 100) * 100),
			0,
			100,
		),
		brandShadow: normalizeHex(
			brandShadow?.hex ?? CUSTOM_THEME_DEFAULTS.brandShadow,
			CUSTOM_THEME_DEFAULTS.brandShadow,
		),
		brandShadowOpacity: clamp(
			Math.round((brandShadow?.alpha ?? CUSTOM_THEME_DEFAULTS.brandShadowOpacity / 100) * 100),
			0,
			100,
		),
		brandGradientStart: normalizeHex(
			brandGradientBg[0]?.hex ?? CUSTOM_THEME_DEFAULTS.brandGradientStart,
			CUSTOM_THEME_DEFAULTS.brandGradientStart,
		),
		brandGradientStartOpacity: clamp(
			Math.round((brandGradientBg[0]?.alpha ?? CUSTOM_THEME_DEFAULTS.brandGradientStartOpacity / 100) * 100),
			0,
			100,
		),
		brandGradientEnd: normalizeHex(
			brandGradientBg.at(-1)?.hex ?? CUSTOM_THEME_DEFAULTS.brandGradientEnd,
			CUSTOM_THEME_DEFAULTS.brandGradientEnd,
		),
		brandGradientEndOpacity: clamp(
			Math.round((brandGradientBg.at(-1)?.alpha ?? CUSTOM_THEME_DEFAULTS.brandGradientEndOpacity / 100) * 100),
			0,
			100,
		),
		brandGradientStrongStart: normalizeHex(
			brandGradientStrongBg[0]?.hex ?? CUSTOM_THEME_DEFAULTS.brandGradientStrongStart,
			CUSTOM_THEME_DEFAULTS.brandGradientStrongStart,
		),
		brandGradientStrongStartOpacity: clamp(
			Math.round(
				(brandGradientStrongBg[0]?.alpha ?? CUSTOM_THEME_DEFAULTS.brandGradientStrongStartOpacity / 100) *
					100,
			),
			0,
			100,
		),
		brandGradientStrongEnd: normalizeHex(
			brandGradientStrongBg.at(-1)?.hex ?? CUSTOM_THEME_DEFAULTS.brandGradientStrongEnd,
			CUSTOM_THEME_DEFAULTS.brandGradientStrongEnd,
		),
		brandGradientStrongEndOpacity: clamp(
			Math.round(
				(brandGradientStrongBg.at(-1)?.alpha ??
					CUSTOM_THEME_DEFAULTS.brandGradientStrongEndOpacity / 100) * 100,
			),
			0,
			100,
		),
		brandGradientBorderColor: normalizeHex(
			brandGradientBorder?.hex ?? CUSTOM_THEME_DEFAULTS.brandGradientBorderColor,
			CUSTOM_THEME_DEFAULTS.brandGradientBorderColor,
		),
		brandGradientBorderOpacity: clamp(
			Math.round(
				(brandGradientBorder?.alpha ?? CUSTOM_THEME_DEFAULTS.brandGradientBorderOpacity / 100) * 100,
			),
			0,
			100,
		),
		loadingBarEnd: normalizeHex(
			loadingBarGradient.at(-1)?.hex ?? CUSTOM_THEME_DEFAULTS.loadingBarEnd,
			CUSTOM_THEME_DEFAULTS.loadingBarEnd,
		),
		success: readThemeHex(styles, '--color-green', CUSTOM_THEME_DEFAULTS.success, '--color-brand'),
		warning: readThemeHex(styles, '--color-orange', CUSTOM_THEME_DEFAULTS.warning, '--color-brand'),
		danger: readThemeHex(styles, '--color-red', CUSTOM_THEME_DEFAULTS.danger, '--color-brand'),
		info: readThemeHex(styles, '--color-blue', CUSTOM_THEME_DEFAULTS.info, '--color-brand'),
		utility: readThemeHex(styles, '--color-purple', CUSTOM_THEME_DEFAULTS.utility, '--color-brand'),
		brandTintStrength: clamp(Math.round((brandHighlight?.alpha ?? 0.18) * 100), 4, 40),
		glassTint: normalizeHex(glassBg?.hex ?? CUSTOM_THEME_DEFAULTS.glassTint, CUSTOM_THEME_DEFAULTS.glassTint),
		glassBorderColor: normalizeHex(
			glassBorder?.hex ?? CUSTOM_THEME_DEFAULTS.glassBorderColor,
			CUSTOM_THEME_DEFAULTS.glassBorderColor,
		),
		glassOpacity: clamp(Math.round((glassBg?.alpha ?? 0.7) * 100), 10, 100),
		glassStrongOpacity: clamp(Math.round((glassBgStrong?.alpha ?? 0.86) * 100), 10, 100),
		glassBorderOpacity: clamp(Math.round((glassBorder?.alpha ?? 0.08) * 100), 0, 40),
	}

	probe.remove()
	return palette
}

export function cloneCustomTheme(theme: CustomTheme) {
	const copy = structuredClone(theme)
	copy.id = makeThemeId()
	copy.name = `${theme.name} Copy`
	copy.createdAt = new Date().toISOString()
	copy.updatedAt = copy.createdAt
	return copy
}

export function serializeCustomTheme(theme: CustomTheme) {
	return JSON.stringify(theme, null, 2)
}

export async function downloadCustomTheme(theme: CustomTheme) {
	if (typeof window === 'undefined') return false

	const safeName = theme.name.toLowerCase().replace(/[^a-z0-9-_]+/gi, '-')
	const path = await save({
		title: 'Export theme',
		defaultPath: `${safeName || 'theme'}.revoria-theme.json`,
		filters: [
			{
				name: 'Revoria Theme',
				extensions: ['revoria-theme.json', 'json'],
			},
		],
	})

	if (!path || Array.isArray(path)) return false

	await invoke('plugin:utils|write_text_file', {
		path,
		content: serializeCustomTheme(theme),
	})

	return true
}
