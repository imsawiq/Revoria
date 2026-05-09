import { convertFileSrc } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'

import {
	applyCustomThemeToDocument,
	clearAppliedCustomTheme,
	type CustomTheme,
	loadStoredActiveCustomThemeId,
	loadStoredCustomThemes,
	persistActiveCustomThemeId,
	persistCustomThemes,
} from '@/helpers/custom-themes'

export const DEFAULT_FEATURE_FLAGS = {
	project_background: false,
	page_path: false,
	worlds_tab: false,
	worlds_in_home: true,
	servers_in_app: false,
}

export const THEME_OPTIONS = [
	'dark',
	'light',
	'system',
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
] as const

export type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS
export type FeatureFlags = Record<FeatureFlag, boolean>
export type ColorTheme = (typeof THEME_OPTIONS)[number]

export type ThemeStore = {
	selectedTheme: ColorTheme
	customThemes: CustomTheme[]
	activeCustomThemeId: string | null
	advancedRendering: boolean
	toggleSidebar: boolean
	glassBlur: number
	glassBorderOpacity: number
	backgroundEffect: 'off' | 'snow' | 'stars' | 'rain'
	backgroundEffectIntensity: number
	pageBackgroundPath: string
	pageBackgroundUrl: string | null
	pageBackgroundOpacity: number

	devMode: boolean
	featureFlags: FeatureFlags
}

const THEME_STORAGE_KEY = 'revoria.theme'

const LIGHT_BASE_THEMES: readonly ColorTheme[] = ['light', 'rose-gold', 'cherry-blossom']

function getStoredTheme(): ColorTheme | null {
	try {
		if (typeof window === 'undefined') return null
		const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
		if (!stored) return null
		return (THEME_OPTIONS as readonly string[]).includes(stored) ? (stored as ColorTheme) : null
	} catch {
		return null
	}
}

export const DEFAULT_THEME_STORE: ThemeStore = {
	selectedTheme: getStoredTheme() ?? 'dark',
	customThemes: loadStoredCustomThemes(),
	activeCustomThemeId: loadStoredActiveCustomThemeId(),
	advancedRendering: true,
	toggleSidebar: false,
	glassBlur: 20,
	glassBorderOpacity: 0.075,
	backgroundEffect: 'off',
	backgroundEffectIntensity: 100,
	pageBackgroundPath: '',
	pageBackgroundUrl: null,
	pageBackgroundOpacity: 0.22,

	devMode: false,
	featureFlags: DEFAULT_FEATURE_FLAGS,
}

function parseCssColor(value: string): { r: number; g: number; b: number; a: number } | null {
	const v = value.trim()
	const rgba = v.match(/^rgba?\((\d+)[,\s]+(\d+)[,\s]+(\d+)(?:[,\s/]+([0-9.]+))?/i)
	if (!rgba) return null
	return {
		r: Number(rgba[1]),
		g: Number(rgba[2]),
		b: Number(rgba[3]),
		a: rgba[4] == null ? 1 : Number(rgba[4]),
	}
}

export const useTheming = defineStore('themeStore', {
	state: () => DEFAULT_THEME_STORE,
	actions: {
		setThemeState(newTheme: ColorTheme, options?: { preserveCustom?: boolean }) {
			if (THEME_OPTIONS.includes(newTheme)) {
				this.selectedTheme = newTheme
			} else {
				this.selectedTheme = 'dark'
			}

			if (!options?.preserveCustom) {
				this.activeCustomThemeId = null
				persistActiveCustomThemeId(null)
				clearAppliedCustomTheme()
			}

			try {
				if (typeof window !== 'undefined') {
					window.localStorage.setItem(THEME_STORAGE_KEY, this.selectedTheme)
				}
			} catch {
				// ignore
			}

			this.setThemeClass()
		},
		setThemeClass() {
			for (const theme of THEME_OPTIONS) {
				document.getElementsByTagName('html')[0].classList.remove(`${theme}-mode`)
			}
			document.getElementsByTagName('html')[0].classList.remove('dark-mode')
			document.getElementsByTagName('html')[0].classList.remove('light-mode')
			document.getElementsByTagName('html')[0].classList.remove('dark')

			let resolvedBaseTheme: 'dark' | 'light' = 'dark'
			if (this.selectedTheme === 'system') {
				const darkThemeMq = window.matchMedia('(prefers-color-scheme: dark)')
				resolvedBaseTheme = darkThemeMq.matches ? 'dark' : 'light'
			} else if (LIGHT_BASE_THEMES.includes(this.selectedTheme)) {
				resolvedBaseTheme = 'light'
			}

			document.getElementsByTagName('html')[0].classList.add(`${resolvedBaseTheme}-mode`)
			if (resolvedBaseTheme === 'dark') {
				document.getElementsByTagName('html')[0].classList.add('dark')
			}
			if (this.selectedTheme !== 'system' && this.selectedTheme !== resolvedBaseTheme) {
				document.getElementsByTagName('html')[0].classList.add(`${this.selectedTheme}-mode`)
			}

			this.applyGlassSettings()
			this.syncCustomThemeOverlay()
		},
		syncCustomThemeOverlay() {
			const activeCustomTheme = this.customThemes.find((theme) => theme.id === this.activeCustomThemeId)
			if (activeCustomTheme) {
				applyCustomThemeToDocument(activeCustomTheme)
			} else {
				clearAppliedCustomTheme()
			}
		},
		applyGlassSettings() {
			const root = document.getElementsByTagName('html')[0]
			const blur = Math.max(0, Math.min(72, this.glassBlur))
			root.style.setProperty('--glass-blur', `${blur}px`)

			const styles = getComputedStyle(root)
			const borderBase =
				parseCssColor(styles.getPropertyValue('--glass-border-base')) ??
				parseCssColor(styles.getPropertyValue('--glass-border'))
			if (borderBase) {
				const opacity = Math.max(0, Math.min(1, this.glassBorderOpacity))
				root.style.setProperty(
					'--glass-border',
					`rgba(${borderBase.r}, ${borderBase.g}, ${borderBase.b}, ${opacity})`,
				)
			}

			const blurFactor = blur / 72
			const bgBase =
				parseCssColor(styles.getPropertyValue('--glass-bg-base')) ??
				parseCssColor(styles.getPropertyValue('--glass-bg'))
			const bgStrongBase =
				parseCssColor(styles.getPropertyValue('--glass-bg-strong-base')) ??
				parseCssColor(styles.getPropertyValue('--glass-bg-strong'))

			if (bgBase) {
				const alpha = Math.max(0.14, Math.min(0.95, bgBase.a - blurFactor * 0.38))
				const value = `rgba(${bgBase.r}, ${bgBase.g}, ${bgBase.b}, ${alpha})`
				root.style.setProperty('--glass-bg', value)
				root.style.setProperty('--color-glass-bg', value)
			}

			if (bgStrongBase) {
				const alpha = Math.max(0.16, Math.min(0.98, bgStrongBase.a - blurFactor * 0.46))
				const value = `rgba(${bgStrongBase.r}, ${bgStrongBase.g}, ${bgStrongBase.b}, ${alpha})`
				root.style.setProperty('--glass-bg-strong', value)
				root.style.setProperty('--color-glass-bg-strong', value)
			}

			this.syncCustomThemeOverlay()
		},
		setGlassBlur(value: number) {
			this.glassBlur = value
			this.applyGlassSettings()
		},
		setGlassBorderOpacity(value: number) {
			this.glassBorderOpacity = value
			this.applyGlassSettings()
		},
		setBackgroundEffect(effect: 'off' | 'snow' | 'stars' | 'rain') {
			this.backgroundEffect = effect
		},
		setBackgroundEffectIntensity(value: number) {
			this.backgroundEffectIntensity = Math.max(10, Math.min(400, Math.round(value)))
		},
		setPageBackground(path: string | null | undefined, opacity?: number) {
			const normalizedPath = typeof path === 'string' ? path.trim() : ''
			this.pageBackgroundPath = normalizedPath
			this.pageBackgroundUrl = normalizedPath ? convertFileSrc(normalizedPath) : null

			if (typeof opacity === 'number') {
				this.pageBackgroundOpacity = Math.max(0, Math.min(1, opacity))
			}
		},
		setPageBackgroundOpacity(value: number) {
			this.pageBackgroundOpacity = Math.max(0, Math.min(1, value))
		},
		getFeatureFlag(key: FeatureFlag) {
			return this.featureFlags[key] ?? DEFAULT_FEATURE_FLAGS[key]
		},
		restoreActiveCustomTheme() {
			const activeCustomTheme = this.customThemes.find((theme) => theme.id === this.activeCustomThemeId)
			if (!activeCustomTheme) {
				this.activeCustomThemeId = null
				persistActiveCustomThemeId(null)
				clearAppliedCustomTheme()
				return
			}

			this.setThemeState(activeCustomTheme.baseTheme, { preserveCustom: true })
		},
		saveCustomThemes(themes: CustomTheme[]) {
			this.customThemes = themes
			persistCustomThemes(themes)

			if (!themes.some((theme) => theme.id === this.activeCustomThemeId)) {
				this.activeCustomThemeId = null
				persistActiveCustomThemeId(null)
				clearAppliedCustomTheme()
			} else {
				this.syncCustomThemeOverlay()
			}
		},
		activateCustomTheme(id: string) {
			const theme = this.customThemes.find((entry) => entry.id === id)
			if (!theme) return
			this.activeCustomThemeId = id
			persistActiveCustomThemeId(id)
			this.setThemeState(theme.baseTheme, { preserveCustom: true })
		},
		clearCustomThemeSelection() {
			this.activeCustomThemeId = null
			persistActiveCustomThemeId(null)
			clearAppliedCustomTheme()
			this.setThemeClass()
		},
		getActiveCustomTheme() {
			return this.customThemes.find((theme) => theme.id === this.activeCustomThemeId) ?? null
		},
		getThemeOptions() {
			return THEME_OPTIONS
		},
	},
})
