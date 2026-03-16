import 'floating-vue/dist/style.css'

import * as Sentry from '@sentry/vue'
import { VueScanPlugin } from '@taijased/vue-render-tracker'
import { VueQueryPlugin } from '@tanstack/vue-query'
import { createPlugin } from '@vintl/vintl/plugin'
import FloatingVue from 'floating-vue'
import { createPinia } from 'pinia'
import { createApp, watch } from 'vue'
import { useStorage } from '@vueuse/core'

import App from '@/App.vue'
import router from '@/routes'
import allEnMessages from '@/locales/combined/en-US.json'
import allRuMessages from '@/locales/combined/ru-RU.json'
import allUkMessages from '@/locales/combined/uk-UA.json'
import allDeDeMessages from '@/locales/combined/de-DE.json'
import allRoMessages from '@/locales/combined/ro-RO.json'

const launcherLanguage = useStorage('launcher-language', 'en')
const localeMap = {
	en: 'en-US',
	ru: 'ru-RU',
	uk: 'uk-UA',
	de: 'de-DE',
	ro: 'ro-RO',
}
const initialLocale = localeMap[launcherLanguage.value] ?? 'en-US'
const normalizeMessages = (messages) =>
	Object.fromEntries(
		Object.entries(messages ?? {}).map(([key, value]) => [
			key,
			value?.message ?? value?.defaultMessage ?? value,
		]),
	)

const VIntlPlugin = createPlugin({
	controllerOpts: {
		defaultLocale: 'en-US',
		locale: initialLocale,
		locales: [
			{
				tag: 'en-US',
				meta: {
					displayName: 'American English',
				},
			},
			{
				tag: 'ru-RU',
				meta: {
					displayName: 'Русский',
				},
			},
			{
				tag: 'uk-UA',
				meta: {
					displayName: 'Українська',
				},
			},
			{
				tag: 'de-DE',
				meta: {
					displayName: 'Deutsch (Deutschland)',
				},
			},
			{
				tag: 'ro-RO',
				meta: {
					displayName: 'Română',
				},
			},
		],
	},
	globalMixin: true,
	injectInto: [],
})

const vintlController = VIntlPlugin.getOrCreateController()
vintlController.addEventListener('localeload', (event) => {
	const tag = event.locale.tag
	const messagesByLocale = {
		'en-US': allEnMessages,
		'ru-RU': allRuMessages,
		'uk-UA': allUkMessages,
		'de-DE': allDeDeMessages,
		'ro-RO': allRoMessages,
	}
	const messages = messagesByLocale[tag] ?? allEnMessages
	event.addMessages(normalizeMessages(messages))
})

const vueScan = new VueScanPlugin({
	enabled: false, // Enable or disable the tracker
	showOverlay: true, // Show overlay to visualize renders
	log: false, // Log render events to the console
	playSound: false, // Play sound on each render
})

const pinia = createPinia()

let app = createApp(App)

Sentry.init({
	app,
	dsn: '', // Disabled — original Modrinth Sentry DSN returns 403
	integrations: [Sentry.browserTracingIntegration({ router })],
	tracesSampleRate: 0.1,
	enabled: false,
})

app.use(VueQueryPlugin)
app.use(vueScan)
app.use(router)
app.use(pinia)
app.use(FloatingVue, {
	themes: {
		'ribbit-popout': {
			$extend: 'dropdown',
			placement: 'bottom-end',
			instantMove: true,
			distance: 8,
		},
	},
})
app.use(VIntlPlugin)

let lastLocale = initialLocale
watch(
	launcherLanguage,
	async (language) => {
		const nextLocale = localeMap[language] ?? 'en-US'
		if (nextLocale === lastLocale) return
		lastLocale = nextLocale
		await vintlController.changeLocale(nextLocale)
	},
	{ immediate: false },
)

app.mount('#app')
