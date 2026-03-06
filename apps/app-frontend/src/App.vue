<script setup>
import { AuthFeature, PanelVersionFeature, TauriModrinthClient } from '@modrinth/api-client'
import {
	ChangeSkinIcon,
	CompassIcon,
	DatabaseIcon,
	ExternalIcon,
	HomeIcon,
	LeftArrowIcon,
	LibraryIcon,
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
	NotificationPanel,
	OverflowMenu,
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
import { $fetch } from 'ofetch'
import { computed, nextTick, onMounted, onUnmounted, provide, ref } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import ModrinthLoadingIndicator from '@/components/LoadingIndicatorBar.vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import FriendsList from '@/components/ui/friends/FriendsList.vue'
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import AppSettingsModal from '@/components/ui/modal/AppSettingsModal.vue'
import AuthGrantFlowWaitModal from '@/components/ui/modal/AuthGrantFlowWaitModal.vue'
import UpdateToast from '@/components/ui/UpdateToast.vue'
import NavButton from '@/components/ui/NavButton.vue'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import { useCheckDisableMouseover } from '@/composables/macCssFix.js'
import { debugAnalytics, optOutAnalytics, trackEvent } from '@/helpers/analytics'
import { check_reachable } from '@/helpers/auth.js'
import { get_user } from '@/helpers/cache.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { useFetch } from '@/helpers/fetch.js'
import { cancelLogin, get as getCreds, login, logout } from '@/helpers/mr_auth.ts'
import { list } from '@/helpers/profile.js'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { get_opening_command, initialize_state } from '@/helpers/state'
import {
	areUpdatesEnabled,
	enqueueUpdateForInstallation,
	getOS,
	getUpdateSize,
	isDev,
	isNetworkMetered,
} from '@/helpers/utils.js'
import {
	provideAppUpdateDownloadProgress,
	subscribeToDownloadProgress,
} from '@/providers/download-progress.ts'
import { useError } from '@/store/error.js'
import { useInstall } from '@/store/install.js'
import { useLoading, useTheming } from '@/store/state'

import { create_profile_and_install_from_file } from './helpers/pack'
import { generateSkinPreviews } from './helpers/rendering/batch-skin-renderer'
import { get_available_capes, get_available_skins } from './helpers/skins'
import { AppNotificationManager } from './providers/app-notifications'

// [AR] Imports
import { get, set } from '@/helpers/settings.ts'

const themeStore = useTheming()

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
const availableSurvey = ref(false)

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

const stateInitialized = ref(false)

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
	await useCheckDisableMouseover()

	document.querySelector('body').addEventListener('click', handleClick)
	document.querySelector('body').addEventListener('auxclick', handleAuxClick)

	checkUpdates()
})

onUnmounted(async () => {
	document.querySelector('body').removeEventListener('click', handleClick)
	document.querySelector('body').removeEventListener('auxclick', handleAuxClick)

	await unlistenUpdateDownload?.()
})

const { formatMessage } = useVIntl()
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
	navLibrary: {
		id: 'app.nav.library',
		defaultMessage: 'Library',
	},
	navCreateInstance: {
		id: 'app.nav.create-instance',
		defaultMessage: 'Create new instance',
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
		onboarded,
		default_page,
		toggle_sidebar,
		developer_mode,
		feature_flags,
		pending_update_toast_for_version,
	} = settingsObj

	if (default_page === 'Library') {
		await router.push('/library')
	}

	os.value = await getOS()
	const dev = await isDev()
	const version = await getVersion()
	showOnboarding.value = !onboarded

	nativeDecorations.value = native_decorations
	if (os.value !== 'MacOS') await getCurrentWindow().setDecorations(native_decorations)

	themeStore.setThemeState(theme)
	if (settingsObj.theme !== themeStore.selectedTheme) {
		settingsObj.theme = themeStore.selectedTheme
		await setSettings(settingsObj)
	}
	themeStore.collapsedNavigation = collapsed_navigation
	themeStore.advancedRendering = advanced_rendering
	themeStore.toggleSidebar = toggle_sidebar
	themeStore.devMode = developer_mode
	themeStore.featureFlags = feature_flags

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
	} else {
		// Other commands are URL-based (deep linking)
		urlModal.value.show(e)
	}
}

const appUpdateDownload = {
	progress: ref(0),
	version: ref(),
}
let unlistenUpdateDownload

const metered = ref(true)
const finishedDownloading = ref(false)
const availableUpdate = ref(null)
const updateSize = ref(null)

async function checkUpdates() {
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

	appUpdateDownload.progress.value = 0
	finishedDownloading.value = false
	availableUpdate.value = update
	updateSize.value = await getUpdateSize(update.rid)
	metered.value = await isNetworkMetered()

	if (!metered.value) {
		await downloadUpdate(update)
	}

	setTimeout(() => {
		void checkUpdates()
	}, 5 * 60 * 1000)
}

async function downloadAvailableUpdate() {
	return await downloadUpdate(availableUpdate.value)
}

async function downloadUpdate(versionToDownload) {
	if (!versionToDownload) {
		handleError('Failed to download update: no version available')
		return
	}

	if (appUpdateDownload.progress.value !== 0 && appUpdateDownload.progress.value < 1) {
		console.log(`Update ${versionToDownload.version} already downloading`)
		return
	}

	try {
		unlistenUpdateDownload = await subscribeToDownloadProgress(
			appUpdateDownload,
			versionToDownload.version,
		)
		await enqueueUpdateForInstallation(versionToDownload.rid)
		finishedDownloading.value = true
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
		surveys = await $fetch('https://api.modrinth.com/v2/surveys')
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
		<UpdateToast
			v-if="availableUpdate && (metered || finishedDownloading)"
			:version="availableUpdate.version"
			:size="updateSize"
			:metered="metered"
			@download="downloadAvailableUpdate"
			@restart="installUpdate"
			@close="availableUpdate = null"
		/>
		<Suspense>
			<AppSettingsModal ref="settingsModal" />
		</Suspense>
		<Suspense>
			<AuthGrantFlowWaitModal ref="modrinthLoginFlowWaitModal" @flow-cancel="cancelLogin" />
		</Suspense>
		<Suspense>
			<InstanceCreationModal ref="installationModal" />
		</Suspense>
		<div class="app-grid-navbar flex flex-col p-3 pt-2 gap-2 w-[--left-bar-width]">
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
			<NavButton
				v-tooltip.right="formatMessage(messages.navLibrary)"
				to="/library"
				:is-subpage="
					() =>
						route.path.startsWith('/instance') ||
						((route.path.startsWith('/browse') || route.path.startsWith('/project')) &&
							route.query.i)
				"
			>
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
				v-tooltip.right="formatMessage(commonMessages.settingsLabel)"
				:to="() => $refs.settingsModal.show()"
			>
				<SettingsIcon />
			</NavButton>
			<OverflowMenu
				v-if="credentials"
				v-tooltip.right="formatMessage(messages.accountMenu)"
				class="w-10 h-10 text-primary rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast border-0 cursor-pointer"
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
	</div>
	<div
		v-if="stateInitialized"
		class="app-contents experimental-styles-within"
		:class="{
			'sidebar-enabled': sidebarVisible,
			'disable-advanced-rendering': !themeStore.advancedRendering,
		}"
	>
		<div class="app-viewport flex-grow router-view">
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
					<div class="p-3 pr-2 flex flex-col gap-3">
						<div
							class="rounded-xl bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] p-3"
						>
							<h3 class="text-base text-primary font-medium m-0">
								{{ formatMessage(messages.sidebarPlayingAs) }}
							</h3>
							<suspense>
								<AccountsCard ref="accounts" mode="small" />
							</suspense>
						</div>
						<div
							class="rounded-xl bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] p-3"
						>
							<h3 class="text-base text-primary font-medium m-0">
								{{ formatMessage(messages.sidebarFriends) }}
							</h3>
							<suspense>
								<FriendsList
									:credentials="credentials"
									:sign-in="() => signIn()"
									:refresh-credentials="fetchCredentials"
									:hide-heading="true"
								/>
							</suspense>
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
		radial-gradient(1200px 700px at 20% 10%, rgba(27, 217, 106, 0.1), transparent 55%),
		radial-gradient(1000px 600px at 80% 25%, rgba(255, 255, 255, 0.06), transparent 60%),
		var(--color-raised-bg);
	height: 100vh;
}

.app-grid-navbar {
	grid-area: nav;
	position: absolute;
	top: calc(var(--top-bar-height) + 2 * var(--shell-gap));
	left: var(--shell-gap);
	bottom: var(--shell-gap);
	width: var(--left-bar-width);
	border-radius: var(--radius-xl);
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
}

.app-grid-statusbar {
	grid-area: status;
	position: absolute;
	top: var(--shell-gap);
	left: var(--shell-gap);
	right: var(--shell-gap);
	height: var(--top-bar-height);
	border-radius: var(--radius-xl);
	background: var(--color-glass-bg);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
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
	background: var(--color-glass-bg);
	border-radius: var(--radius-xl);
	border: 1px solid var(--glass-border);
	box-shadow: var(--glass-shadow);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
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
	background: var(--color-glass-bg);
	backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
	-webkit-backdrop-filter: blur(var(--glass-blur)) saturate(var(--glass-saturate));
}

.app-viewport {
	flex-grow: 1;
	height: 100%;
	overflow: auto;
	overflow-x: hidden;
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

.app-page-swap-shell {
	position: relative;
	will-change: transform, opacity, filter;
}

.app-page-swap-enter-active,
.app-page-swap-leave-active {
	transition:
		opacity 280ms cubic-bezier(0.2, 0.8, 0.2, 1),
		transform 340ms cubic-bezier(0.16, 1, 0.3, 1),
		filter 280ms ease;
}

.app-page-swap-enter-from {
	opacity: 0;
	transform: translateY(14px) scale(0.985);
	filter: blur(4px);
}

.app-page-swap-leave-to {
	opacity: 0;
	transform: translateY(-8px) scale(0.992);
	filter: blur(3px);
}

.revoria-update-alert {
	border-color: color-mix(in srgb, var(--color-brand) 72%, var(--glass-border));
	box-shadow:
		0 0 0 1px color-mix(in srgb, var(--color-brand) 48%, transparent),
		0 0 18px color-mix(in srgb, var(--color-brand) 28%, transparent);
	animation: revoria-update-pulse 1.25s ease-in-out infinite;
}

@keyframes revoria-update-pulse {
	0%,
	100% {
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--color-brand) 44%, transparent),
			0 0 12px color-mix(in srgb, var(--color-brand) 20%, transparent);
	}
	50% {
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--color-brand) 74%, transparent),
			0 0 28px color-mix(in srgb, var(--color-brand) 38%, transparent);
	}
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
