import {
	CoffeeIcon,
	GameIcon,
	GaugeIcon,
	GlobeIcon,
	LanguagesIcon,
	PaintbrushIcon,
	PaletteIcon,
	ReportIcon,
	ShieldIcon,
} from '@modrinth/assets'
import { defineMessage } from '@vintl/vintl'

import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import ThemesSettings from '@/components/ui/settings/ThemesSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import LanguageSettings from '@/components/ui/settings/LanguageSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import ProxySettings from '@/components/ui/settings/ProxySettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'

export const appSettingsTabs = [
	{
		id: 'appearance',
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Внешний вид',
		}),
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		id: 'themes',
		name: defineMessage({
			id: 'app.settings.tabs.themes',
			defaultMessage: 'Темы',
		}),
		icon: PaletteIcon,
		content: ThemesSettings,
	},
	{
		id: 'language',
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Язык',
		}),
		icon: LanguagesIcon,
		content: LanguageSettings,
	},
	{
		id: 'privacy',
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Конфиденциальность',
		}),
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		id: 'java-installations',
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Установки Java',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		id: 'default-instance-options',
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Предустановки',
		}),
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		id: 'resource-management',
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Управление данными',
		}),
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		id: 'proxy',
		name: defineMessage({
			id: 'app.settings.tabs.proxy',
			defaultMessage: 'Прокси',
		}),
		icon: GlobeIcon,
		content: ProxySettings,
	},
	{
		id: 'feature-flags',
		name: defineMessage({
			id: 'app.settings.tabs.feature-flags',
			defaultMessage: 'Флаги функций',
		}),
		icon: ReportIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
] as const

export type AppSettingsTabId = (typeof appSettingsTabs)[number]['id']
