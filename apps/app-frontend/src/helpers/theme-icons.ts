import { invoke } from '@tauri-apps/api/core'

import amethystIcon from '@/assets/theme-icons/amethyst.png?url'
import auroraIcon from '@/assets/theme-icons/aurora.png?url'
import cherryBlossomIcon from '@/assets/theme-icons/cherry-blossom.png?url'
import defaultIcon from '@/assets/theme-icons/default.png?url'
import obsidianGoldIcon from '@/assets/theme-icons/obsidian-gold.png?url'
import oledIcon from '@/assets/theme-icons/oled.png?url'
import retroIcon from '@/assets/theme-icons/retro.png?url'
import roseGoldIcon from '@/assets/theme-icons/rose-gold.png?url'
import sapphireIcon from '@/assets/theme-icons/sapphire.png?url'
import sunsetIcon from '@/assets/theme-icons/sunset.png?url'
import whiteIcon from '@/assets/theme-icons/white.png?url'
import type { ColorTheme } from '@/store/theme'

const THEME_ICON_URLS: Record<Exclude<ColorTheme, 'system'>, string> = {
	dark: defaultIcon,
	light: whiteIcon,
	oled: oledIcon,
	retro: retroIcon,
	sapphire: sapphireIcon,
	amethyst: amethystIcon,
	sunset: sunsetIcon,
	aurora: auroraIcon,
	nord: defaultIcon,
	'cherry-cola': defaultIcon,
	slate: defaultIcon,
	'rose-gold': roseGoldIcon,
	'obsidian-gold': obsidianGoldIcon,
	'cherry-blossom': cherryBlossomIcon,
}

let lastAppliedThemeKey: string | null = null

export function resolveThemeIconKey(
	theme: ColorTheme,
	prefersDark = false,
): Exclude<ColorTheme, 'system'> {
	if (theme === 'system') {
		return prefersDark ? 'dark' : 'light'
	}

	return theme
}

export function getThemeIconUrl(theme: ColorTheme, prefersDark = false): string {
	return THEME_ICON_URLS[resolveThemeIconKey(theme, prefersDark)]
}

export async function applyLauncherWindowIcon(
	theme: ColorTheme,
	prefersDark = false,
): Promise<void> {
	const iconKey = `${resolveThemeIconKey(theme, prefersDark)}:${prefersDark ? 'dark' : 'light'}`
	if (lastAppliedThemeKey === iconKey) {
		return
	}

	await invoke('plugin:utils|set_theme_window_icon', {
		theme,
		prefersDark,
	})

	lastAppliedThemeKey = iconKey
}
