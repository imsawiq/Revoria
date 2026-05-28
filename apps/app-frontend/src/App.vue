<script setup>
import { AuthFeature, PanelVersionFeature, TauriModrinthClient } from '@modrinth/api-client'
import {
	ChangeSkinIcon,
	CompassIcon,
	DatabaseIcon,
	DiscordIcon,
	ExternalIcon,
	HomeIcon,
	LeftArrowIcon,
	LibraryIcon,
	LinkIcon,
	LogInIcon,
	LogOutIcon,
	MaximizeIcon,
	MinimizeIcon,
	NotepadTextIcon,
	PlusIcon,
	RestoreIcon,
	RightArrowIcon,
	ServerIcon,
	SettingsIcon,
	UserIcon,
	WorldIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonStyled,
	commonMessages,
	CreationFlowModal,
	NotificationPanel,
	OverflowMenu,
	provideI18n,
	provideModrinthClient,
	provideNotificationManager,
	providePageContext,
	useDebugLogger,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import { computed, nextTick, onMounted, onUnmounted, provide, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import ModrinthLoadingIndicator from '@/components/LoadingIndicatorBar.vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import BackgroundEffects from '@/components/ui/BackgroundEffects.vue'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import FriendsList from '@/components/ui/friends/FriendsList.vue'
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import AuthGrantFlowWaitModal from '@/components/ui/modal/AuthGrantFlowWaitModal.vue'
import NavButton from '@/components/ui/NavButton.vue'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import UpdateToast from '@/components/ui/UpdateToast.vue'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import { useCheckDisableMouseover } from '@/composables/macCssFix.js'
import { debugAnalytics, optOutAnalytics, trackEvent } from '@/helpers/analytics'
import { check_reachable } from '@/helpers/auth.js'
import { get_user } from '@/helpers/cache.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { useFetch } from '@/helpers/fetch.js'
import { cancelLogin, get as getCreds, login, logout } from '@/helpers/mr_auth.ts'
import { list, run as runProfile } from '@/helpers/profile.js'
import { get, get as getSettings, set, set as setSettings } from '@/helpers/settings.ts'
import { get_opening_command, initialize_state } from '@/helpers/state'
import { applyLauncherWindowIcon } from '@/helpers/theme-icons'
import {
	areUpdatesEnabled,
	enqueueUpdateForInstallation,
	getOS,
	getUpdateSize,
	isDev,
	isNetworkMetered,
	showLauncherLogsFolder,
} from '@/helpers/utils.js'
import allDeDeMessages from '@/locales/combined/de-DE.json'
import allEnMessages from '@/locales/combined/en-US.json'
import allRoMessages from '@/locales/combined/ro-RO.json'
import allRuMessages from '@/locales/combined/ru-RU.json'
import allUkMessages from '@/locales/combined/uk-UA.json'
import { createContentInstall, provideContentInstall } from '@/providers/content-install'
import {
	provideAppUpdateDownloadProgress,
	subscribeToDownloadProgress,
} from '@/providers/download-progress.ts'
import { setupProviders } from '@/providers/setup'
import { useError } from '@/store/error.js'
import { useInstall } from '@/store/install.js'
import { useLoading, useTheming } from '@/store/state'

import { create_profile_and_install_from_file } from './helpers/pack'
import { generateSkinPreviews } from './helpers/rendering/batch-skin-renderer'
import { get_available_capes, get_available_skins } from './helpers/skins'
import { AppNotificationManager } from './providers/app-notifications'

const themeStore = useTheming()

themeStore.setThemeClass()

const systemThemeMediaQuery =
	typeof window !== 'undefined' ? window.matchMedia('(prefers-color-scheme: dark)') : null

async function syncLauncherThemeIcon() {
	try {
		await applyLauncherWindowIcon(themeStore.selectedTheme, systemThemeMediaQuery?.matches ?? false)
	} catch (error) {
		console.warn('Failed to apply themed launcher icon', error)
	}
}

const notificationManager = new AppNotificationManager()
provideNotificationManager(notificationManager)
const { handleError, addNotification } = notificationManager

const tauriApiClient = new TauriModrinthClient({
	userAgent: `modrinth/theseus/${getVersion()} (support@modrinth.com)`,
	features: [
		new AuthFeature({
			token: async () => (await getCreds()).session,
		}),
		new PanelVersionFeature(),
	],
})
provideModrinthClient(tauriApiClient)
providePageContext({
	hierarchicalSidebarAvailable: ref(true),
	showAds: ref(false),
})

const stateInitialized = ref(false)

const {
	installationModal,
	fetchExistingInstanceNames,
	handleCreate,
	handleBrowseModpacks,
	searchModpacks,
	getProjectVersions,
} = setupProviders(notificationManager, { stateReady: stateInitialized })

const launcherLanguage = useStorage('launcher-language', 'en')
const languageToLocale = {
	en: 'en-US',
	ru: 'ru-RU',
	uk: 'uk-UA',
	de: 'de-DE',
	ro: 'ro-RO',
}
const i18nLocale = ref(languageToLocale[launcherLanguage.value] ?? 'en-US')
const availableSurvey = ref(false)

async function syncDiscordRpcLanguage(language) {
	try {
		await invoke('plugin:utils|set_discord_rpc_language', { language })
	} catch (error) {
		console.warn('Failed to sync Discord RPC language', error)
	}
}

const urlModal = ref(null)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const showOnboarding = ref(false)
const nativeDecorations = ref(false)

const os = ref('')

const criticalErrorMessage = ref()

const isMaximized = ref(false)

const authUnreachableDebug = useDebugLogger('AuthReachableChecker')
const authServerQuery = useQuery({
	queryKey: ['authServerReachability'],
	queryFn: async () => {
		await check_reachable()
		authUnreachableDebug('Auth servers are reachable')
		return true
	},
	refetchInterval: 5 * 60 * 1000, // 5 minutes
	retry: false,
	refetchOnWindowFocus: false,
})

const authUnreachable = computed(() => {
	if (authServerQuery.isError.value && !authServerQuery.isLoading.value) {
		console.warn('Failed to reach auth servers', authServerQuery.error.value)
		return true
	}
	return false
})

onMounted(async () => {
	const currentWindow = getCurrentWindow()
	await currentWindow.show()
	await currentWindow.setFocus()
	await useCheckDisableMouseover()
	await syncLauncherThemeIcon()
	await syncDiscordRpcLanguage(launcherLanguage.value)

	document.querySelector('body').addEventListener('click', handleClick)
	document.querySelector('body').addEventListener('auxclick', handleAuxClick)
	window.addEventListener('keydown', handleDebugShortcut)

	checkUpdates()
})

onUnmounted(async () => {
	document.querySelector('body').removeEventListener('click', handleClick)
	document.querySelector('body').removeEventListener('auxclick', handleAuxClick)
	window.removeEventListener('keydown', handleDebugShortcut)
	if (updateCheckTimeout !== null) {
		clearTimeout(updateCheckTimeout)
		updateCheckTimeout = null
	}
	await unlistenUpdateDownload?.()
})

async function handleDebugShortcut(event) {
	if (!event.ctrlKey || event.key !== 'F12') return
	event.preventDefault()
	await showLauncherLogsFolder().catch(handleError)
}

watch(
	() => themeStore.selectedTheme,
	() => {
		syncLauncherThemeIcon()
	},
)

if (systemThemeMediaQuery) {
	const handleSystemThemeChange = () => {
		if (themeStore.selectedTheme === 'system') {
			themeStore.setThemeClass()
			syncLauncherThemeIcon()
		}
	}

	systemThemeMediaQuery.addEventListener('change', handleSystemThemeChange)

	onUnmounted(() => {
		systemThemeMediaQuery.removeEventListener('change', handleSystemThemeChange)
	})
}

const vintl = useVIntl()
const { formatMessage } = vintl
const normalizeMessages = (messages) =>
	Object.fromEntries(
		Object.entries(messages ?? {}).map(([key, value]) => [
			key,
			value?.message ?? value?.defaultMessage ?? value,
		]),
	)
const allMessagesByLocale = {
	'en-US': allEnMessages,
	'ru-RU': allRuMessages,
	'uk-UA': allUkMessages,
	'de-DE': allDeDeMessages,
	'ro-RO': allRoMessages,
}

Object.entries(allMessagesByLocale).forEach(([locale, messages]) => {
	vintl.addMessages(locale, normalizeMessages(messages))
})
let langSwitchTimeout
function triggerLanguageTransition() {
	const root = document.documentElement
	root.classList.add('lang-switching')
	// Force reflow so the transition applies reliably.
	void root.offsetHeight
	root.classList.add('lang-switching-active')
	if (langSwitchTimeout) {
		clearTimeout(langSwitchTimeout)
	}
	langSwitchTimeout = window.setTimeout(() => {
		root.classList.remove('lang-switching-active')
		root.classList.remove('lang-switching')
	}, 260)
}
watch(
	launcherLanguage,
	async (lang) => {
		await syncDiscordRpcLanguage(lang)
		triggerLanguageTransition()
		const nextLocale = languageToLocale[lang] ?? 'en-US'
		i18nLocale.value = nextLocale
	},
	{ immediate: true },
)
provideI18n({
	locale: i18nLocale,
	t: (key, values) => formatMessage({ id: key, defaultMessage: key }, values),
	setLocale: async (locale) => {
		i18nLocale.value = locale
		await vintl.changeLocale(locale)
	},
})
const messages = defineMessages({
	updateInstalledToastTitle: {
		id: 'app.update.complete-toast.title',
		defaultMessage: 'Version {version} was successfully installed!',
	},
	updateInstalledToastText: {
		id: 'app.update.complete-toast.text',
		defaultMessage: 'Click here to view the changelog.',
	},
	reloadToUpdate: {
		id: 'app.update.reload-to-update',
		defaultMessage: 'Reload to install update',
	},
	downloadUpdate: {
		id: 'app.update.download-update',
		defaultMessage: 'Download update',
	},
	downloadingUpdate: {
		id: 'app.update.downloading-update',
		defaultMessage: 'Downloading update ({percent}%)',
	},
	authUnreachableHeader: {
		id: 'app.auth-servers.unreachable.header',
		defaultMessage: 'Cannot reach authentication servers',
	},
	authUnreachableBody: {
		id: 'app.auth-servers.unreachable.body',
		defaultMessage:
			'Minecraft authentication servers may be down right now. Check your internet connection and try again later.',
	},
	navHome: {
		id: 'app.nav.home',
		defaultMessage: 'Home',
	},
	navWorlds: {
		id: 'app.nav.worlds',
		defaultMessage: 'Worlds',
	},
	navServers: {
		id: 'app.nav.servers',
		defaultMessage: 'Servers',
	},
	navDiscover: {
		id: 'app.nav.discover',
		defaultMessage: 'Discover content',
	},
	navSkins: {
		id: 'app.nav.skins',
		defaultMessage: 'Skins (Beta)',
	},
	navMaintenance: {
		id: 'app.nav.maintenance',
		defaultMessage: 'Maintenance',
	},
	navSyncing: {
		id: 'app.nav.syncing',
		defaultMessage: 'Syncing',
	},
	navLibrary: {
		id: 'app.nav.library',
		defaultMessage: 'Library',
	},
	navCreateInstance: {
		id: 'app.nav.create-instance',
		defaultMessage: 'Create new instance',
	},
	navDiscord: {
		id: 'app.nav.discord',
		defaultMessage: 'Discord server',
	},
	accountMenu: {
		id: 'app.account.menu',
		defaultMessage: 'Modrinth account',
	},
	signedInAs: {
		id: 'app.account.signed-in-as',
		defaultMessage: 'Signed in as',
	},
	signOut: {
		id: 'app.account.sign-out',
		defaultMessage: 'Sign out',
	},
	signIn: {
		id: 'app.account.sign-in',
		defaultMessage: 'Sign in to a Modrinth account',
	},
	surveyTitle: {
		id: 'app.survey.title',
		defaultMessage: 'Hey there Modrinth user!',
	},
	surveyBody: {
		id: 'app.survey.body',
		defaultMessage:
			'Would you mind answering a few questions about your experience with Modrinth App?',
	},
	surveyBodySecondary: {
		id: 'app.survey.body.secondary',
		defaultMessage:
			'This feedback will go directly to the Modrinth team and help guide future updates!',
	},
	surveyTake: {
		id: 'app.survey.take',
		defaultMessage: 'Take survey',
	},
	surveyDecline: {
		id: 'app.survey.decline',
		defaultMessage: 'No thanks',
	},
	sidebarPlayingAs: {
		id: 'app.sidebar.playing-as',
		defaultMessage: 'Playing as',
	},
	sidebarFriends: {
		id: 'app.sidebar.friends',
		defaultMessage: 'Friends',
	},
})

async function setupApp() {
	// [AR] Patched
	const settings = await get()
	settings.personalized_ads = false
	settings.telemetry = false
	await set(settings)

	stateInitialized.value = true
	// Trigger loading transition so SplashScreen dismisses
	// (previously triggered by Suspense @pending/@resolve from async page components)
	loading.startLoading()
	await nextTick()
	loading.stopLoading()
	const settingsObj = await getSettings()
	const {
		native_decorations,
		theme,
		telemetry,
		personalized_ads,
		collapsed_navigation,
		advanced_rendering,
		glass_blur,
		glass_border_opacity,
		background_effect,
		background_effect_intensity,
		page_background_path,
		page_background_opacity,
		onboarded,
		default_page,
		toggle_sidebar,
		developer_mode,
		feature_flags,
		pending_update_toast_for_version,
		auto_download_updates,
	} = settingsObj

	if (default_page === 'library') {
		await router.push('/library')
	}

	os.value = await getOS()
	const dev = await isDev()
	const version = await getVersion()
	showOnboarding.value = !onboarded

	nativeDecorations.value = native_decorations
	if (os.value !== 'MacOS') await getCurrentWindow().setDecorations(native_decorations)

	if (themeStore.getActiveCustomTheme()) {
		themeStore.restoreActiveCustomTheme()
	} else {
		themeStore.setThemeState(theme)
	}
	if (settingsObj.theme !== themeStore.selectedTheme) {
		settingsObj.theme = themeStore.selectedTheme
		await setSettings(settingsObj)
	}
	themeStore.collapsedNavigation = collapsed_navigation
	themeStore.advancedRendering = advanced_rendering
	themeStore.glassBlur = glass_blur
	themeStore.glassBorderOpacity = glass_border_opacity
	themeStore.backgroundEffect = background_effect
	themeStore.backgroundEffectIntensity = background_effect_intensity ?? 100
	themeStore.setPageBackground(page_background_path, page_background_opacity ?? 0.22)
	themeStore.applyGlassSettings()
	themeStore.toggleSidebar = toggle_sidebar
	themeStore.devMode = developer_mode
	themeStore.featureFlags = feature_flags
	autoDownloadUpdates.value = auto_download_updates ?? true

	isMaximized.value = await getCurrentWindow().isMaximized()

	await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
	})

	// [AR] Patched
	if (!telemetry) {
		console.info('[AR] • Telemetry disabled by default (Hard patched).')
		optOutAnalytics()
	}
	if (!personalized_ads) {
		console.info('[AR] • Personalized ads disabled by default (Hard patched).')
	}
	if (dev) debugAnalytics()
	trackEvent('Launched', { version, dev, onboarded })

	if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

	const osType = await type()
	if (osType === 'macos') {
		document.getElementsByTagName('html')[0].classList.add('mac')
	} else {
		document.getElementsByTagName('html')[0].classList.add('windows')
	}

	await warning_listener((e) =>
		addNotification({
			title: 'Warning',
			text: e.message,
			type: 'warn',
		}),
	)

	useFetch(
		`https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
		'criticalAnnouncements',
		true,
	)
		.then((response) => response.json())
		.then((res) => {
			if (res && res.header && res.body) {
				criticalErrorMessage.value = res
			}
		})
		.catch(() => {
			console.log(
				`No critical announcement found at https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
			)
		})

	get_opening_command().then(handleCommand)
	fetchCredentials()

	try {
		const skins = (await get_available_skins()) ?? []
		const capes = (await get_available_capes()) ?? []
		generateSkinPreviews(skins, capes)
	} catch (error) {
		console.warn('Failed to generate skin previews in app setup.', error)
	}

	if (pending_update_toast_for_version !== null) {
		addNotification({
			title: formatMessage(messages.updateInstalledToastTitle, {
				version: pending_update_toast_for_version,
			}),
			text: formatMessage(messages.updateInstalledToastText),
			type: 'success',
		})

		const settings = await getSettings()
		settings.pending_update_toast_for_version = null
		await setSettings(settings)
	}

	if (osType === 'windows') {
		await processPendingSurveys()
	} else {
		console.info('Skipping user surveys on non-Windows platforms')
	}
}

const stateFailed = ref(false)
initialize_state()
	.then(() => {
		setupApp().catch((err) => {
			stateFailed.value = true
			console.error(err)
			error.showError(err, null, false, 'state_init')
		})
	})
	.catch((err) => {
		stateFailed.value = true
		console.error('Failed to initialize app', err)
		error.showError(err, null, false, 'state_init')
	})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}

const router = useRouter()
router.afterEach((to, from, failure) => {
	trackEvent('PageView', {
		path: to.path,
		fromPath: from.path,
		failed: failure,
	})
})
const route = useRoute()

const appPageTransitionKey = computed(() => {
	const [section] = route.path.split('/').filter(Boolean)
	return section || 'home'
})

const loading = useLoading()
loading.setEnabled(false)

const error = useError()
const errorModal = ref()

const install = useInstall()
const modInstallModal = ref()
const installConfirmModal = ref()
const incompatibilityWarningModal = ref()

const contentInstall = createContentInstall({ router, handleError })
provideContentInstall(contentInstall)

const credentials = ref()

const modrinthLoginFlowWaitModal = ref()

async function fetchCredentials() {
	const creds = await getCreds().catch(handleError)
	if (creds && creds.user_id) {
		creds.user = await get_user(creds.user_id).catch(handleError)
	}
	credentials.value = creds ?? null
}

async function signIn() {
	modrinthLoginFlowWaitModal.value.show()

	try {
		await login()
		await fetchCredentials()
	} catch (error) {
		if (
			typeof error === 'object' &&
			typeof error['message'] === 'string' &&
			error.message.includes('Login canceled')
		) {
			// Not really an error due to being a result of user interaction, show nothing
		} else {
			handleError(error)
		}
	} finally {
		modrinthLoginFlowWaitModal.value.hide()
	}
}

async function logOut() {
	await logout().catch(handleError)
	await fetchCredentials()
}

const MIDAS_BITFLAG = 1 << 0
const hasPlus = computed(
	() =>
		credentials.value &&
		credentials.value.user &&
		(credentials.value.user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG,
)

const sidebarToggled = ref(true)

themeStore.$subscribe(() => {
	sidebarToggled.value = !themeStore.toggleSidebar
})

const forceSidebar = computed(
	() => route.path.startsWith('/browse') || route.path.startsWith('/project'),
)
const sidebarVisible = computed(() => sidebarToggled.value || forceSidebar.value)

onMounted(() => {
	invoke('show_window')

	error.setErrorModal(errorModal.value)

	install.setIncompatibilityWarningModal(incompatibilityWarningModal)
	install.setInstallConfirmModal(installConfirmModal)
	install.setModInstallModal(modInstallModal)
})

const accounts = ref(null)
provide('accountsCard', accounts)

command_listener(handleCommand)
async function handleCommand(e) {
	if (!e) return

	if (e.event === 'RunMRPack') {
		// RunMRPack should directly install a local mrpack given a path
		if (e.path.endsWith('.mrpack')) {
			await create_profile_and_install_from_file(e.path).catch(handleError)
			trackEvent('InstanceCreate', {
				source: 'CreationModalFileDrop',
			})
		}
	} else if (e.event === 'RunProfile') {
		await runProfile(e.path).catch(handleError)
	} else {
		// Other commands are URL-based (deep linking)
		urlModal.value.show(e)
	}
}

const appUpdateDownloadProgress = ref(0)
const appUpdateDownloadVersion = ref()
const appUpdateDownload = {
	progress: appUpdateDownloadProgress,
	version: appUpdateDownloadVersion,
}
let unlistenUpdateDownload

const metered = ref(true)
const autoDownloadUpdates = ref(true)
const availableUpdate = ref(null)
const updateSize = ref(null)
const updateToastStatus = ref('available')
let updateCheckTimeout = null

function openUpdateToast(status) {
	updateToastStatus.value = status
}

function closeUpdateToast() {
	if (updateToastStatus.value === 'downloading') {
		return
	}
	availableUpdate.value = null
	updateSize.value = null
	appUpdateDownloadProgress.value = 0
}

if (typeof window !== 'undefined') {
	window.__revoriaDebugShowUpdateToast = (overrides = {}) => {
		const next =
			overrides && typeof overrides === 'object' && !Array.isArray(overrides) ? overrides : {}

		availableUpdate.value = {
			version: '1.4.1',
			rid: 0,
			...next,
		}
		updateSize.value = typeof next.size === 'number' ? next.size : 128 * 1024 * 1024
		metered.value = typeof next.metered === 'boolean' ? next.metered : false
		appUpdateDownloadProgress.value = typeof next.progress === 'number' ? next.progress : 0.42
		updateToastStatus.value = typeof next.status === 'string' ? next.status : 'downloading'
	}
}

async function checkUpdates() {
	if (updateCheckTimeout !== null) {
		clearTimeout(updateCheckTimeout)
		updateCheckTimeout = null
	}

	try {
		if (!(await areUpdatesEnabled())) {
			console.log('Skipping update check as updates are disabled in this build or environment')
			return
		}

		const update = await invoke('plugin:updater|check')
		if (!update) {
			console.log('No update available')
			return
		}

		if (update.version === availableUpdate.value?.version) {
			console.log('Update is already known')
			return
		}

		const latestSettings = await getSettings()
		autoDownloadUpdates.value = latestSettings.auto_download_updates ?? true

		appUpdateDownloadProgress.value = 0
		availableUpdate.value = update
		updateSize.value = await getUpdateSize(update.rid).catch((error) => {
			console.warn('Failed to get update size', error)
			return null
		})
		metered.value = await isNetworkMetered().catch((error) => {
			console.warn('Failed to detect metered network, requiring manual update download', error)
			return true
		})
		const shouldAutoDownload = autoDownloadUpdates.value && !metered.value
		openUpdateToast(shouldAutoDownload ? 'downloading' : 'available')

		if (shouldAutoDownload) {
			await downloadUpdate(update)
		}
	} catch (error) {
		console.warn('Failed to check for launcher updates', error)
	} finally {
		updateCheckTimeout = setTimeout(
			() => {
				void checkUpdates()
			},
			5 * 60 * 1000,
		)
	}
}

async function downloadAvailableUpdate() {
	return await downloadUpdate(availableUpdate.value)
}

async function downloadUpdate(versionToDownload) {
	if (!versionToDownload) {
		handleError('Failed to download update: no version available')
		return
	}

	if (appUpdateDownloadProgress.value !== 0 && appUpdateDownloadProgress.value < 1) {
		console.log(`Update ${versionToDownload.version} already downloading`)
		return
	}

	try {
		appUpdateDownloadProgress.value = 0
		openUpdateToast('downloading')
		unlistenUpdateDownload = await subscribeToDownloadProgress(
			appUpdateDownload,
			versionToDownload.version,
		)
		await enqueueUpdateForInstallation(versionToDownload.rid)
		appUpdateDownloadProgress.value = 1
		openUpdateToast('downloaded')
		await unlistenUpdateDownload?.()
		unlistenUpdateDownload = null
	} catch (e) {
		await unlistenUpdateDownload?.()
		unlistenUpdateDownload = null
		handleError(e)
	}
}

async function installUpdate() {
	await handleClose()
}

provideAppUpdateDownloadProgress(appUpdateDownload)

function handleClick(e) {
	let target = e.target
	while (target != null) {
		if (target.matches('a')) {
			if (
				target.href &&
				['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
				!target.classList.contains('router-link-active') &&
				!target.href.startsWith('http://localhost') &&
				!target.href.startsWith('https://tauri.localhost') &&
				!target.href.startsWith('http://tauri.localhost')
			) {
				openUrl(target.href)
			}
			e.preventDefault()
			break
		}
		target = target.parentElement
	}
}

function handleAuxClick(e) {
	// disables middle click -> new tab
	if (e.button === 1) {
		e.preventDefault()
		// instead do a left click
		const event = new MouseEvent('click', {
			view: window,
			bubbles: true,
			cancelable: true,
		})
		e.target.dispatchEvent(event)
	}
}

function cleanupOldSurveyDisplayData() {
	const threeWeeksAgo = new Date()
	threeWeeksAgo.setDate(threeWeeksAgo.getDate() - 21)

	for (let i = 0; i < localStorage.length; i++) {
		const key = localStorage.key(i)

		if (key.startsWith('survey-') && key.endsWith('-display')) {
			const dateValue = new Date(localStorage.getItem(key))
			if (dateValue < threeWeeksAgo) {
				localStorage.removeItem(key)
			}
		}
	}
}

async function openSurvey() {
	if (!availableSurvey.value) {
		console.error('No survey to open')
		return
	}

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const formId = availableSurvey.value.tally_id

	const popupOptions = {
		layout: 'modal',
		width: 700,
		autoClose: 2000,
		hideTitle: true,
		hiddenFields: {
			user_id: userId,
		},
		onOpen: () => console.info('Opened user survey'),
		onClose: () => {
			console.info('Closed user survey')
		},
		onSubmit: () => console.info('Active user survey submitted'),
	}

	try {
		if (window.Tally?.openPopup) {
			console.info(`Opening Tally popup for user survey (form ID: ${formId})`)
			dismissSurvey()
			window.Tally.openPopup(formId, popupOptions)
		} else {
			console.warn('Tally script not yet loaded')
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
	}

	console.info(`Found user survey to show with tally_id: ${formId}`)
	window.Tally.openPopup(formId, popupOptions)
}

function dismissSurvey() {
	localStorage.setItem(`survey-${availableSurvey.value.id}-display`, new Date())
	availableSurvey.value = undefined
}

async function processPendingSurveys() {
	function isWithinLastTwoWeeks(date) {
		const twoWeeksAgo = new Date()
		twoWeeksAgo.setDate(twoWeeksAgo.getDate() - 14)
		return date >= twoWeeksAgo
	}

	cleanupOldSurveyDisplayData()

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const instances = await list().catch(handleError)
	const isActivePlayer =
		instances.findIndex(
			(instance) =>
				isWithinLastTwoWeeks(instance.last_played) && !isWithinLastTwoWeeks(instance.created),
		) >= 0

	let surveys = []
	try {
		surveys = await invoke('plugin:utils|proxy_get_json', {
			url: 'https://api.modrinth.com/v2/surveys',
			headers: {
				Accept: 'application/json',
			},
		})
	} catch (e) {
		console.error('Error fetching surveys:', e)
	}

	const surveyToShow = surveys.find(
		(survey) =>
			!!(
				localStorage.getItem(`survey-${survey.id}-display`) === null &&
				survey.type === 'tally_app' &&
				((survey.condition === 'active_player' && isActivePlayer) ||
					(survey.assigned_users?.includes(userId) && !survey.dismissed_users?.includes(userId)))
			),
	)

	if (surveyToShow) {
		availableSurvey.value = surveyToShow
	} else {
		console.info('No user survey to show')
	}
}
</script>

<template>
	<SplashScreen v-if="!stateFailed" ref="splashScreen" data-tauri-drag-region />
	<div id="teleports"></div>
	<div
		v-if="stateInitialized"
		class="app-grid-layout experimental-styles-within relative"
		:class="{ 'disable-advanced-rendering': !themeStore.advancedRendering }"
	>
		<div data-tauri-drag-region class="app-top-drag-strip"></div>
		<div class="app-effects-layer">
			<BackgroundEffects />
		</div>
		<UpdateToast
			v-if="availableUpdate"
			:version="availableUpdate.version"
			:size="updateSize"
			:metered="metered"
			:progress="appUpdateDownloadProgress"
			:status="updateToastStatus"
			@download="downloadAvailableUpdate"
			@restart="installUpdate"
			@close="closeUpdateToast"
		/>
		<Suspense>
			<AuthGrantFlowWaitModal ref="modrinthLoginFlowWaitModal" @flow-cancel="cancelLogin" />
		</Suspense>
		<CreationFlowModal
			ref="installationModal"
			type="instance"
			show-snapshot-toggle
			:fetch-existing-instance-names="fetchExistingInstanceNames"
			:search-modpacks="searchModpacks"
			:get-project-versions="getProjectVersions"
			@create="handleCreate"
			@browse-modpacks="handleBrowseModpacks"
		/>
		<div class="app-grid-navbar flex flex-col p-3 pt-2 gap-1 w-[--left-bar-width]">
			<NavButton v-tooltip.right="formatMessage(messages.navHome)" to="/">
				<HomeIcon />
			</NavButton>
			<NavButton
				v-if="themeStore.featureFlags.worlds_tab"
				v-tooltip.right="formatMessage(messages.navWorlds)"
				to="/worlds"
			>
				<WorldIcon />
			</NavButton>
			<NavButton
				v-if="themeStore.featureFlags.servers_in_app"
				v-tooltip.right="formatMessage(messages.navServers)"
				to="/hosting/manage"
			>
				<ServerIcon />
			</NavButton>
			<NavButton
				v-tooltip.right="formatMessage(messages.navDiscover)"
				to="/browse/mod"
				:is-primary="() => route.path.startsWith('/browse') && !route.query.i"
				:is-subpage="(route) => route.path.startsWith('/project') && !route.query.i"
			>
				<CompassIcon />
			</NavButton>
			<NavButton v-tooltip.right="formatMessage(messages.navSkins)" to="/skins">
				<ChangeSkinIcon />
			</NavButton>
			<NavButton v-tooltip.right="formatMessage(messages.navMaintenance)" to="/maintenance">
				<DatabaseIcon />
			</NavButton>
			<NavButton v-tooltip.right="formatMessage(messages.navSyncing)" to="/syncing">
				<LinkIcon />
			</NavButton>
			<NavButton v-tooltip.right="formatMessage(messages.navLibrary)" to="/library">
				<LibraryIcon />
			</NavButton>
			<div class="h-px w-8 mx-auto my-2 bg-divider"></div>
			<suspense>
				<QuickInstanceSwitcher />
			</suspense>
			<NavButton
				v-tooltip.right="formatMessage(messages.navCreateInstance)"
				:to="() => $refs.installationModal.show()"
				:disabled="offline"
			>
				<PlusIcon />
			</NavButton>
			<div class="flex flex-grow"></div>
			<NavButton
				v-tooltip.right="formatMessage(messages.navDiscord)"
				:to="() => openUrl('https://discord.gg/Rjt9zZG7Dj')"
			>
				<DiscordIcon />
			</NavButton>
			<NavButton
				v-tooltip.right="formatMessage(commonMessages.settingsLabel)"
				to="/settings"
				:is-primary="(route) => route.path.startsWith('/settings')"
			>
				<SettingsIcon />
			</NavButton>
			<OverflowMenu
				v-if="credentials"
				v-tooltip.right="formatMessage(messages.accountMenu)"
				class="w-10 h-10 text-primary rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-[--color-button-bg-hover] hover:text-contrast border-0 cursor-pointer"
				:options="[
					{
						id: 'view-profile',
						action: () => openUrl('https://modrinth.com/user/' + credentials.user.username),
					},
					{
						id: 'sign-out',
						action: () => logOut(),
						color: 'danger',
					},
				]"
				placement="right-end"
			>
				<Avatar :src="credentials.user.avatar_url" alt="" size="32px" circle />
				<template #view-profile>
					<UserIcon />
					<span class="inline-flex items-center gap-1">
						{{ formatMessage(messages.signedInAs) }}
						<span class="inline-flex items-center gap-1 text-contrast font-semibold">
							<Avatar :src="credentials.user.avatar_url" alt="" size="20px" circle />
							{{ credentials.user.username }}
						</span>
					</span>
					<ExternalIcon />
				</template>
				<template #sign-out> <LogOutIcon /> {{ formatMessage(messages.signOut) }} </template>
			</OverflowMenu>
			<NavButton v-else v-tooltip.right="formatMessage(messages.signIn)" :to="() => signIn()">
				<LogInIcon class="text-brand" />
			</NavButton>
		</div>
		<div data-tauri-drag-region class="app-grid-statusbar h-[--top-bar-height] flex relative">
			<div class="loading-indicator-container">
				<ModrinthLoadingIndicator :height="3" />
			</div>
			<div data-tauri-drag-region class="flex p-3">
				<div data-tauri-drag-region class="flex items-center gap-2 ml-3">
					<button
						class="cursor-pointer p-0 m-0 text-contrast border-none outline-none bg-button-bg rounded-lg flex items-center justify-center w-7 h-7 hover:bg-[--color-button-bg-hover] transition-colors"
						@click="router.back()"
					>
						<LeftArrowIcon />
					</button>
					<button
						class="cursor-pointer p-0 m-0 text-contrast border-none outline-none bg-button-bg rounded-lg flex items-center justify-center w-7 h-7 hover:bg-[--color-button-bg-hover] transition-colors"
						@click="router.forward()"
					>
						<RightArrowIcon />
					</button>
				</div>
				<Breadcrumbs class="pt-[2px]" />
			</div>
			<section data-tauri-drag-region class="flex ml-auto items-center">
				<ButtonStyled
					v-if="!forceSidebar && themeStore.toggleSidebar"
					:type="sidebarToggled ? 'standard' : 'transparent'"
					circular
				>
					<button
						class="mr-3 transition-transform"
						:class="{ 'rotate-180': !sidebarToggled }"
						@click="sidebarToggled = !sidebarToggled"
					>
						<RightArrowIcon />
					</button>
				</ButtonStyled>
				<div class="flex mr-3">
					<Suspense>
						<RunningAppBar />
					</Suspense>
				</div>
				<section v-if="!nativeDecorations" class="window-controls" data-tauri-drag-region-exclude>
					<Button class="titlebar-button" icon-only @click="() => getCurrentWindow().minimize()">
						<MinimizeIcon />
					</Button>
					<Button
						class="titlebar-button"
						icon-only
						@click="() => getCurrentWindow().toggleMaximize()"
					>
						<RestoreIcon v-if="isMaximized" />
						<MaximizeIcon v-else />
					</Button>
					<Button class="titlebar-button close" icon-only @click="handleClose">
						<XIcon />
					</Button>
				</section>
			</section>
		</div>
		<div
			class="app-contents experimental-styles-within"
			:class="{
				'sidebar-enabled': sidebarVisible,
				'disable-advanced-rendering': !themeStore.advancedRendering,
			}"
		>
			<div
				class="app-viewport flex-grow router-view"
				:style="
					themeStore.pageBackgroundUrl
						? {
								backgroundImage: `linear-gradient(
									color-mix(in srgb, var(--color-glass-bg-strong) ${Math.round(
										(1 - themeStore.pageBackgroundOpacity) * 100,
									)}%, transparent),
									color-mix(in srgb, var(--color-glass-bg-strong) ${Math.round(
										(1 - themeStore.pageBackgroundOpacity) * 100,
									)}%, transparent)
								), url(${themeStore.pageBackgroundUrl})`,
								backgroundPosition: 'center',
								backgroundRepeat: 'no-repeat',
								backgroundSize: 'cover',
								backgroundAttachment: 'scroll',
							}
						: undefined
				"
			>
				<transition name="popup-survey">
					<div
						v-if="availableSurvey"
						class="w-[400px] z-20 fixed -bottom-12 pb-16 right-[--right-bar-width] mr-4 rounded-t-2xl bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] border-b-0 p-4"
					>
						<h2 class="text-lg font-extrabold mt-0 mb-2">
							{{ formatMessage(messages.surveyTitle) }}
						</h2>
						<p class="m-0 leading-tight">
							{{ formatMessage(messages.surveyBody) }}
						</p>
						<p class="mt-3 mb-4 leading-tight">
							{{ formatMessage(messages.surveyBodySecondary) }}
						</p>
						<div class="flex gap-2">
							<ButtonStyled color="brand">
								<button @click="openSurvey">
									<NotepadTextIcon /> {{ formatMessage(messages.surveyTake) }}
								</button>
							</ButtonStyled>
							<ButtonStyled>
								<button @click="dismissSurvey">
									<XIcon /> {{ formatMessage(messages.surveyDecline) }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</transition>
				<div
					v-if="themeStore.featureFlags.page_path"
					class="absolute bottom-0 left-0 m-2 bg-tooltip-bg text-tooltip-text font-semibold rounded-full px-2 py-1 text-xs z-50"
				>
					{{ route.fullPath }}
				</div>
				<div
					id="background-teleport-target"
					class="absolute h-full -z-10 rounded-tl-[--radius-xl] overflow-hidden"
					:style="{
						width: 'calc(100% - var(--right-bar-width))',
					}"
				></div>
				<Admonition
					v-if="criticalErrorMessage"
					type="critical"
					:header="criticalErrorMessage.header"
					class="m-6 mb-0"
				>
					<div
						class="markdown-body text-primary"
						v-html="renderString(criticalErrorMessage.body ?? '')"
					></div>
				</Admonition>
				<Admonition
					v-if="authUnreachable"
					type="warning"
					:header="formatMessage(messages.authUnreachableHeader)"
					class="m-6 mb-0"
				>
					{{ formatMessage(messages.authUnreachableBody) }}
				</Admonition>
				<RouterView v-slot="{ Component }">
					<template v-if="Component">
						<Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
							<Transition name="app-page-swap" mode="out-in">
								<div :key="appPageTransitionKey" class="app-page-swap-shell">
									<component :is="Component"></component>
								</div>
							</Transition>
						</Suspense>
					</template>
				</RouterView>
			</div>
			<div
				class="app-sidebar mt-px shrink-0 flex flex-col border-0 border-l border-divider border-solid overflow-auto"
				:class="{ 'has-plus': hasPlus }"
			>
				<div
					class="app-sidebar-scrollable flex-grow shrink overflow-y-auto relative"
					:class="{ 'pb-12': !hasPlus }"
				>
					<div id="sidebar-teleport-target" class="sidebar-teleport-content"></div>
					<div class="sidebar-default-content" :class="{ 'sidebar-enabled': sidebarVisible }">
						<div class="right-panel-shell">
							<section class="right-panel-section right-panel-section-account">
								<div class="right-panel-section-header">
									<h3>{{ formatMessage(messages.sidebarPlayingAs) }}</h3>
								</div>
								<suspense>
									<AccountsCard ref="accounts" mode="small" />
								</suspense>
							</section>
							<section class="right-panel-section right-panel-section-friends">
								<div class="right-panel-section-header">
									<h3>{{ formatMessage(messages.sidebarFriends) }}</h3>
								</div>
								<suspense>
									<FriendsList
										:credentials="credentials"
										:sign-in="() => signIn()"
										:refresh-credentials="fetchCredentials"
										:hide-heading="true"
									/>
								</suspense>
							</section>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
	<URLConfirmModal ref="urlModal" />
	<NotificationPanel has-sidebar />
	<ErrorModal ref="errorModal" />
	<ModInstallModal ref="modInstallModal" />
	<IncompatibilityWarningModal ref="incompatibilityWarningModal" />
	<InstallConfirmModal ref="installConfirmModal" />
</template>

<style lang="scss" scoped>
@use '../../../packages/assets/styles/neon-icon.scss' as *;
@use '../../../packages/assets/styles/neon-text.scss' as *;
.window-controls {
	z-index: 20;
	display: none;
	flex-direction: row;
	align-items: center;

	.titlebar-button {
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all ease-in-out 0.1s;
		background-color: transparent;
		color: var(--color-base);
		height: 100%;
		width: 3rem;
		position: relative;
		box-shadow: none;

		&:last-child {
			padding-right: 0.75rem;
			width: 3.75rem;
		}

		svg {
			width: 1.25rem;
			height: 1.25rem;
		}

		&::before {
			content: '';
			border-radius: 999999px;
			width: 3rem;
			height: 3rem;
			aspect-ratio: 1 / 1;
			margin-block: auto;
			position: absolute;
			background-color: transparent;
			scale: 0.9;
			transition: all ease-in-out 0.2s;
			z-index: -1;
		}

		&.close {
			&:hover,
			&:active {
				color: var(--color-accent-contrast);

				&::before {
					background-color: var(--color-red);
				}
			}
		}

		&:hover,
		&:active {
			color: var(--color-contrast);

			&::before {
				background-color: var(--color-button-bg);
				scale: 1;
			}
		}
	}
}

.app-grid-layout,
.app-contents {
	--top-bar-height: 3rem;
	--left-bar-width: 4rem;
	--right-bar-width: 260px;
	--shell-gap: 0.75rem;
}

.app-grid-layout {
	display: grid;
	grid-template: 'status status' 'nav dummy';
	grid-template-columns: auto 1fr;
	grid-template-rows: auto 1fr;
	position: relative;
	background:
		radial-gradient(
			1000px 620px at 18% 8%,
			color-mix(in srgb, var(--color-brand) 12%, transparent),
			transparent 58%
		),
		radial-gradient(
			860px 520px at 84% 16%,
			color-mix(in srgb, var(--color-blue) 8%, transparent),
			transparent 62%
		),
		linear-gradient(180deg, var(--color-bg) 0%, var(--color-raised-bg) 100%);
	height: 100vh;
}

.app-top-drag-strip {
	position: absolute;
	top: 0;
	left: 0;
	right: 0;
	height: var(--shell-gap);
	z-index: 2;
	-webkit-app-region: drag;
}

.app-effects-layer {
	position: absolute;
	inset: 0;
	z-index: 0;
	overflow: hidden;
}

.app-grid-navbar {
	grid-area: nav;
	position: absolute;
	z-index: 1;
	top: calc(var(--top-bar-height) + 2 * var(--shell-gap));
	left: var(--shell-gap);
	bottom: var(--shell-gap);
	width: var(--left-bar-width);
	border-radius: var(--radius-lg);
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
}

.app-grid-statusbar {
	grid-area: status;
	position: absolute;
	z-index: 1;
	top: var(--shell-gap);
	left: var(--shell-gap);
	right: var(--shell-gap);
	height: var(--top-bar-height);
	border-radius: var(--radius-lg);
	background: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
	overflow: hidden;
	contain: paint;
}

[data-tauri-drag-region-exclude] {
	-webkit-app-region: no-drag;
}

.app-contents {
	position: absolute;
	z-index: 1;
	left: calc(var(--left-bar-width) + 2 * var(--shell-gap));
	top: calc(var(--top-bar-height) + 2 * var(--shell-gap));
	right: var(--shell-gap);
	bottom: var(--shell-gap);
	background: var(--color-glass-bg-strong);
	border-radius: var(--radius-lg);
	border: 1px solid var(--glass-border);
	overflow: hidden;

	display: grid;
	grid-template-columns: 1fr 0px;
	transition: grid-template-columns 0.4s ease-in-out;

	&.sidebar-enabled {
		grid-template-columns: 1fr var(--right-bar-width);
	}
}

.loading-indicator-container {
	position: absolute;
	top: 0;
	left: 0;
	right: 0;
	height: 3px;
	border-radius: 0;
	overflow: hidden;
	pointer-events: none;
}

.app-sidebar {
	overflow: visible;
	width: var(--right-bar-width);
	position: relative;
	height: calc(100vh - var(--top-bar-height));
	background: linear-gradient(
		180deg,
		color-mix(in srgb, var(--color-raised-bg) 46%, transparent) 0%,
		color-mix(in srgb, var(--color-glass-bg) 84%, transparent) 44%,
		color-mix(in srgb, var(--color-bg) 72%, transparent) 100%
	);
	border-left-color: color-mix(in srgb, var(--color-divider) 64%, transparent) !important;
}

.app-viewport {
	flex-grow: 1;
	height: 100%;
	overflow: auto;
	overflow-x: hidden;
	position: relative;
	box-sizing: border-box;
	padding-bottom: 1.5rem;
}

.app-page-swap-shell,
#background-teleport-target {
	z-index: 1;
}

.app-page-swap-shell {
	position: relative;
	will-change: transform, opacity;
	min-height: 100%;
}

.sidebar-teleport-content {
	display: contents;
}

.sidebar-default-content {
	display: none;
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}

.right-panel-shell {
	display: flex;
	flex-direction: column;
	gap: 0.9rem;
	min-width: 0;
	padding: 1rem 0.75rem 1.2rem 0.8rem;
}

.right-panel-section {
	min-width: 0;
	padding: 0.85rem;
	border-radius: 0.85rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 72%, transparent);
	background: linear-gradient(
		180deg,
		color-mix(in srgb, var(--color-glass-bg-strong) 66%, transparent) 0%,
		color-mix(in srgb, var(--color-glass-bg) 38%, transparent) 100%
	);
	box-shadow:
		inset 0 1px 0 color-mix(in srgb, white 5%, transparent),
		0 8px 22px color-mix(in srgb, black 18%, transparent);
	transition:
		transform 220ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
		background 220ms cubic-bezier(0.22, 1, 0.36, 1),
		box-shadow 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.right-panel-section:hover {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 18%, var(--glass-border) 82%);
	box-shadow:
		inset 0 1px 0 color-mix(in srgb, white 6%, transparent),
		0 11px 28px color-mix(in srgb, black 21%, transparent);
}

.right-panel-section-account {
	padding-bottom: 0.75rem;
}

.right-panel-section-friends {
	background: linear-gradient(
		180deg,
		color-mix(in srgb, var(--color-raised-bg) 24%, transparent) 0%,
		color-mix(in srgb, var(--color-glass-bg) 36%, transparent) 100%
	);
	box-shadow: inset 0 1px 0 color-mix(in srgb, white 4%, transparent);
}

.right-panel-section-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	margin-bottom: 0.65rem;
}

.right-panel-section-header h3 {
	margin: 0;
	color: var(--color-secondary);
	font-size: 0.72rem;
	font-weight: 800;
	line-height: 1.1;
	letter-spacing: 0.045em;
	text-transform: uppercase;
	transition: color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.right-panel-section:hover .right-panel-section-header h3 {
	color: color-mix(in srgb, var(--color-secondary) 76%, var(--color-contrast) 24%);
}

.sidebar-default-content :deep(.accounts-shell > .button-base.mt-2) {
	margin-top: 0;
	min-height: 3.15rem;
	padding: 0.55rem 0.65rem;
	border-radius: 0.75rem;
	background: linear-gradient(
		180deg,
		color-mix(in srgb, var(--color-button-bg) 74%, var(--color-glass-bg-strong) 26%) 0%,
		color-mix(in srgb, var(--color-button-bg) 58%, transparent) 100%
	);
	border-color: color-mix(in srgb, var(--glass-border) 84%, transparent);
	box-shadow: none;
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		background 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
		box-shadow 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.sidebar-default-content :deep(.accounts-shell > .button-base.mt-2:hover) {
	transform: translateY(-1px);
	border-color: color-mix(in srgb, var(--color-brand) 26%, var(--glass-border) 74%);
	background: linear-gradient(
		180deg,
		color-mix(in srgb, var(--color-button-bg-hover) 76%, var(--color-glass-bg-strong) 24%) 0%,
		color-mix(in srgb, var(--color-button-bg-hover) 58%, transparent) 100%
	);
	box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-brand) 10%, transparent);
}

.sidebar-default-content :deep(.accounts-shell > .button-base.mt-2:active) {
	transform: translateY(0) scale(0.985);
}

.sidebar-default-content :deep(.account-title) {
	font-size: 0.9rem;
	font-weight: 750;
}

.sidebar-default-content :deep(.account-subtitle) {
	font-size: 0.7rem;
	color: var(--color-secondary);
}

.sidebar-default-content :deep(.friends-search-bar) {
	height: 2.05rem;
	padding: 0.45rem 0.55rem;
	border-width: 1px !important;
	border-radius: 0.65rem;
	background: color-mix(in srgb, var(--color-button-bg) 42%, transparent);
}

.sidebar-default-content :deep(.rounded-xl.bg-\[--color-glass-bg-strong\]) {
	border-radius: 0.75rem;
	border-color: color-mix(in srgb, var(--glass-border) 70%, transparent);
	background: color-mix(in srgb, var(--color-button-bg) 38%, transparent);
	box-shadow: none;
	padding: 0.85rem;
	transition:
		background 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.sidebar-default-content :deep(.rounded-xl.bg-\[--color-glass-bg-strong\]:hover) {
	border-color: color-mix(in srgb, var(--color-brand) 18%, var(--glass-border) 82%);
	background: color-mix(in srgb, var(--color-button-bg-hover) 44%, transparent);
}

.sidebar-default-content :deep(.rounded-xl.bg-\[--color-glass-bg-strong\] .w-9.h-9) {
	width: 2rem;
	height: 2rem;
	border-radius: 0.65rem;
}

.sidebar-default-content :deep(.rounded-xl.bg-\[--color-glass-bg-strong\] .text-sm) {
	color: var(--color-secondary);
	font-size: 0.79rem;
	line-height: 1.25;
}

.sidebar-default-content :deep(.rounded-xl.bg-\[--color-glass-bg-strong\] .text-brand) {
	color: var(--color-brand);
}

.sidebar-default-content :deep(.accordion) {
	min-width: 0;
}

.sidebar-default-content :deep(.accordion button) {
	border-radius: 0.65rem;
	transition:
		background-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.sidebar-default-content :deep(.accordion button:active) {
	transform: scale(0.985);
}

.sidebar-default-content :deep(.accordion h3) {
	color: var(--color-secondary);
	font-size: 0.78rem;
	font-weight: 750;
}

.sidebar-default-content :deep(.accordion [class*='grid-cols-']) {
	margin-left: 0.35rem;
	margin-right: 0;
	padding: 0.22rem 0.28rem;
	border-radius: 0.75rem;
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		background-color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.sidebar-default-content :deep(.accordion [class*='grid-cols-']:hover) {
	transform: translateX(2px);
	background: color-mix(in srgb, var(--color-button-bg-hover) 72%, transparent);
}

.sidebar-default-content :deep(.accordion .w-12.h-12) {
	width: 2rem;
	height: 2rem;
}

.sidebar-default-content :deep(.accordion .text-sm) {
	font-size: 0.8rem;
	font-weight: 650;
}

.sidebar-default-content :deep(.accordion .text-xs) {
	color: var(--color-secondary);
	font-size: 0.68rem;
}

.popup-survey-enter-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.51, 1.08, 0.35, 1.15);
	transform-origin: top center;
}

.popup-survey-leave-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.68, -0.17, 0.23, 0.11);
	transform-origin: top center;
}

.popup-survey-enter-from,
.popup-survey-leave-to {
	opacity: 0;
	transform: translateY(10rem) scale(0.8) scaleY(1.6);
}

.toast-enter-active {
	transition: opacity 0.25s linear;
}

.toast-enter-from,
.toast-leave-to {
	opacity: 0;
}

.app-page-swap-enter-active,
.app-page-swap-leave-active {
	transition:
		opacity 280ms cubic-bezier(0.2, 0.8, 0.2, 1),
		transform 340ms cubic-bezier(0.16, 1, 0.3, 1);
}

.app-page-swap-enter-from {
	opacity: 0;
	transform: translateY(14px) scale(0.988);
}

.app-page-swap-leave-to {
	opacity: 0;
	transform: translateY(-8px) scale(0.994);
}

.revoria-update-alert {
	border-color: var(--color-brand);
	box-shadow: 0 0 0 2px var(--color-brand-highlight);
	animation: none;
}

@media (prefers-reduced-motion: no-preference) {
	.toast-enter-active,
	.nav-button-animated-enter-active {
		transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
	}

	.toast-leave-active,
	.nav-button-animated-leave-active {
		transition: all 0.25s ease;
	}

	.toast-enter-from {
		scale: 0.5;
		translate: 0 -10rem;
		opacity: 0;
	}

	.toast-leave-to {
		scale: 0.96;
		translate: 20rem 0;
		opacity: 0;
	}

	.nav-button-animated-enter-active {
		position: relative;
	}

	.nav-button-animated-enter-active::before {
		content: '';
		inset: 0;
		border-radius: 100vw;
		background-color: var(--color-brand-highlight);
		position: absolute;
		animation: pop 0.5s ease-in forwards;
		opacity: 0;
	}

	@keyframes pop {
		0% {
			scale: 0.5;
		}
		50% {
			opacity: 0.5;
		}
		100% {
			scale: 1.5;
		}
	}

	.nav-button-animated-enter-from {
		scale: 0.5;
		translate: -2rem 0;
		opacity: 0;
	}

	.nav-button-animated-leave-to {
		scale: 0.75;
		opacity: 0;
	}

	.fade-enter-active {
		transition: 0.25s ease-in-out;
	}

	.fade-enter-from {
		opacity: 0;
	}
}
</style>
<style>
html.lang-switching .app-grid-layout,
html.lang-switching .app-contents {
	transition:
		opacity 220ms ease,
		filter 220ms ease;
}

html.lang-switching.lang-switching-active .app-grid-layout,
html.lang-switching.lang-switching-active .app-contents {
	opacity: 0.82;
	filter: none;
}

@media (prefers-reduced-motion: reduce) {
	html.lang-switching .app-grid-layout,
	html.lang-switching .app-contents {
		transition: none;
	}
	html.lang-switching.lang-switching-active .app-grid-layout,
	html.lang-switching.lang-switching-active .app-contents {
		opacity: 1;
		filter: none;
	}
}

.mac {
	.app-grid-statusbar {
		padding-left: 5rem;
	}
}

.windows {
	.fake-appbar {
		height: 2.5rem !important;
	}

	.window-controls {
		display: flex !important;
	}

	.info-card {
		right: 8rem;
	}

	.profile-card {
		right: 8rem;
	}
}
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
