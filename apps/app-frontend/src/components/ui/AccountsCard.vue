<template>
	<div
		v-if="mode !== 'isolated'"
		ref="button"
		class="button-base mt-2 px-3 py-2 bg-button-bg rounded-xl flex items-center gap-2"
		:class="{ expanded: mode === 'expanded' }"
		@click="toggleMenu"
	>
		<Avatar
			size="36px"
			:src="
				selectedAccount ? avatarUrl : 'https://launcher-files.modrinth.com/assets/steve_head.png'
			"
		/>
		<div class="flex flex-col w-full">
			<span class="account-title">
				<component
					:is="getAccountType(selectedAccount)"
					v-if="selectedAccount"
					class="vector-icon"
				/>
				{{ selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount) }}
			</span>
			<span class="text-secondary text-xs">{{ formatMessage(messages.minecraftAccount) }}</span>
		</div>
		<DropdownIcon class="dropdown-icon shrink-0" />
	</div>
	<transition name="fade">
		<Card
			v-if="showCard || mode === 'isolated'"
			ref="card"
			class="account-card"
			:class="{ expanded: mode === 'expanded', isolated: mode === 'isolated' }"
		>
			<div v-if="selectedAccount" class="selected account">
				<Avatar size="xs" :src="avatarUrl" />
				<div>
					<h4>
						<component :is="getAccountType(selectedAccount)" class="vector-icon" />
						{{ selectedAccount.profile.name }}
					</h4>
					<p>{{ formatMessage(messages.selected) }}</p>
				</div>
				<Button
					v-tooltip="formatMessage(messages.logOut)"
					icon-only
					color="raised"
					@click="logout(selectedAccount.profile.id)"
				>
					<TrashIcon />
				</Button>
			</div>
			<div v-else class="login-section account">
				<h4>{{ formatMessage(messages.notSignedIn) }}</h4>
				<Button
					v-tooltip="formatMessage(messages.logViaMicrosoft)"
					:disabled="microsoftLoginDisabled"
					icon-only
					@click="login()"
				>
					<MicrosoftIcon v-if="!microsoftLoginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</Button>
				<Button
					v-tooltip="formatMessage(messages.addOfflineAccount)"
					icon-only
					@click="showOfflineLoginModal()"
				>
					<PirateIcon />
				</Button>
				<Button
					v-tooltip="formatMessage(messages.logViaElyBy)"
					icon-only
					@click="showElybyLoginModal()"
				>
					<ElyByIcon v-if="!elybyLoginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</Button>
			</div>
			<div v-if="displayAccounts.length > 0" class="account-group">
				<div v-for="account in displayAccounts" :key="account.profile.id" class="account-row">
					<Button class="option account" @click="setAccount(account)">
						<Avatar :src="getAccountAvatarUrl(account)" class="icon" />
						<p class="account-type">
							<component :is="getAccountType(account)" class="vector-icon" />
							{{ account.profile.name }}
						</p>
					</Button>
					<Button
						v-tooltip="formatMessage(messages.logOut)"
						icon-only
						@click="logout(account.profile.id)"
					>
						<TrashIcon />
					</Button>
				</div>
			</div>
			<div v-if="accounts.length > 0" class="login-section account centered">
				<Button v-tooltip="formatMessage(messages.logViaMicrosoft)" icon-only @click="login()">
					<MicrosoftIcon v-if="!microsoftLoginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</Button>
				<Button
					v-tooltip="formatMessage(messages.addOfflineAccount)"
					icon-only
					@click="showOfflineLoginModal()"
				>
					<PirateIcon />
				</Button>
				<Button
					v-tooltip="formatMessage(messages.logViaElyBy)"
					icon-only
					@click="showElybyLoginModal()"
				>
					<ElyByIcon v-if="!elybyLoginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
				</Button>
			</div>
		</Card>
	</transition>
	<ModalWrapper
		ref="addElybyModal"
		class="modal"
		:header="formatMessage(messages.authenticateElyBy)"
	>
		<ModalWrapper
			ref="requestElybyTwoFactorCodeModal"
			class="modal"
			:header="formatMessage(messages.elyByRequested2fa)"
		>
			<div class="flex flex-col gap-4 px-6 py-5">
				<label class="label">{{ formatMessage(messages.enter2faCode) }}</label>
				<input
					v-model="elybyTwoFactorCode"
					type="text"
					:placeholder="formatMessage(messages.twoFactorPlaceholder)"
					class="input"
				/>
				<div class="mt-6 ml-auto">
					<Button icon-only color="primary" class="continue-button" @click="addElybyProfile()">
						{{ formatMessage(messages.continueAction) }}
					</Button>
				</div>
			</div>
		</ModalWrapper>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="label">{{ formatMessage(messages.enterPlayerNameOrEmail) }}</label>
			<input
				v-model="elybyLogin"
				type="text"
				:placeholder="formatMessage(messages.playerNameOrEmailPlaceholder)"
				class="input"
			/>
			<label class="label">{{ formatMessage(messages.enterPassword) }}</label>
			<input
				v-model="elybyPassword"
				type="password"
				:placeholder="formatMessage(messages.passwordPlaceholder)"
				class="input"
			/>
			<div class="mt-6 ml-auto">
				<Button icon-only color="primary" class="continue-button" @click="addElybyProfile()">
					{{ formatMessage(messages.login) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="addOfflineModal"
		class="modal"
		:header="formatMessage(messages.addOfflineAccountHeader)"
	>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="label">{{ formatMessage(messages.enterPlayerName) }}</label>
			<input
				v-model="offlinePlayerName"
				type="text"
				:placeholder="formatMessage(messages.playerNamePlaceholder)"
				class="input"
			/>
			<div class="mt-6 ml-auto">
				<Button icon-only color="primary" class="continue-button" @click="addOfflineProfile()">
					{{ formatMessage(messages.login) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="authenticationElybyErrorModal"
		class="modal"
		:header="formatMessage(messages.errorAuthElyByHeader)"
	>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="text-base font-medium text-red-700">
				{{ formatMessage(messages.errorWhileLoggingIn) }}
			</label>

			<div class="mt-6 ml-auto">
				<Button color="primary" class="retry-button" @click="retryAddElybyProfile">
					{{ formatMessage(messages.tryAgain) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="inputElybyErrorModal"
		class="modal"
		:header="formatMessage(messages.errorInputElyByHeader)"
	>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="text-base font-medium text-red-700">
				{{ formatMessage(messages.errorAddingElyByAccount) }}
			</label>

			<ul class="list-disc list-inside text-sm space-y-1">
				<li>{{ formatMessage(messages.checkPlayerNameOrEmail) }}</li>
				<li>{{ formatMessage(messages.checkPassword) }}</li>
			</ul>

			<div class="mt-6 ml-auto">
				<Button color="primary" class="retry-button" @click="retryAddElybyProfile">
					{{ formatMessage(messages.tryAgain) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="inputErrorModal"
		class="modal"
		:header="formatMessage(messages.errorInputOfflineHeader)"
	>
		<div class="flex flex-col gap-4 px-6 py-5">
			<label class="text-base font-medium text-red-700">
				{{ formatMessage(messages.errorAddingOfflineAccount) }}
			</label>

			<ul class="list-disc list-inside text-sm space-y-1">
				<li>{{ formatMessage(messages.checkPlayerName) }}</li>
				<li>
					{{
						formatMessage(messages.playerNameLengthRule, {
							min: minOfflinePlayerNameLength,
							max: maxOfflinePlayerNameLength,
						})
					}}
				</li>
			</ul>

			<div class="mt-6 ml-auto">
				<Button color="primary" class="retry-button" @click="retryAddOfflineProfile">
					{{ formatMessage(messages.tryAgain) }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="exceptionErrorModal"
		class="modal"
		:header="formatMessage(messages.unexpectedErrorHeader)"
	>
		<div class="modal-body">
			<label class="label">{{ formatMessage(messages.unexpectedErrorBody) }}</label>
		</div>
	</ModalWrapper>
</template>

<script setup>
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import {
	elyby_auth_authenticate,
	elyby_login,
	get_default_user,
	login as login_flow,
	offline_login,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { process_listener } from '@/helpers/events'
import { getPlayerHeadUrl } from '@/helpers/rendering/batch-skin-renderer.ts'
import { get_available_skins } from '@/helpers/skins'
import { handleSevereError } from '@/store/error.js'
import {
	DropdownIcon,
	ElyByIcon as Elyby,
	ElyByIcon,
	MicrosoftIcon as License,
	MicrosoftIcon,
	PirateIcon as Offline,
	PirateIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import { Avatar, Button, Card, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, onBeforeUnmount, onMounted, onUnmounted, ref } from 'vue'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	selectAccount: { id: 'accounts-card.select-account', defaultMessage: 'Select account' },
	minecraftAccount: { id: 'accounts-card.minecraft-account', defaultMessage: 'Minecraft account' },
	selected: { id: 'accounts-card.selected', defaultMessage: 'Selected' },
	logOut: { id: 'accounts-card.log-out', defaultMessage: 'Log out' },
	notSignedIn: { id: 'accounts-card.not-signed-in', defaultMessage: 'Not signed in' },
	logViaMicrosoft: { id: 'accounts-card.log-via-microsoft', defaultMessage: 'Log via Microsoft' },
	addOfflineAccount: {
		id: 'accounts-card.add-offline-account',
		defaultMessage: 'Add offline account',
	},
	logViaElyBy: { id: 'accounts-card.log-via-elyby', defaultMessage: 'Log via Ely.by' },
	authenticateElyBy: {
		id: 'accounts-card.authenticate-elyby',
		defaultMessage: 'Authenticate with Ely.by',
	},
	elyByRequested2fa: {
		id: 'accounts-card.elyby-requested-2fa',
		defaultMessage: 'Ely.by requested 2FA code for authentication',
	},
	enter2faCode: { id: 'accounts-card.enter-2fa-code', defaultMessage: 'Enter your 2FA code' },
	twoFactorPlaceholder: {
		id: 'accounts-card.2fa-placeholder',
		defaultMessage: 'Your 2FA code here...',
	},
	continueAction: { id: 'action.continue', defaultMessage: 'Continue' },
	enterPlayerNameOrEmail: {
		id: 'accounts-card.enter-player-name-or-email',
		defaultMessage: 'Enter your player name or email (preferred)',
	},
	playerNameOrEmailPlaceholder: {
		id: 'accounts-card.player-name-or-email-placeholder',
		defaultMessage: 'Your player name or email here...',
	},
	enterPassword: { id: 'accounts-card.enter-password', defaultMessage: 'Enter your password' },
	passwordPlaceholder: {
		id: 'accounts-card.password-placeholder',
		defaultMessage: 'Your password here...',
	},
	login: { id: 'auth.login', defaultMessage: 'Login' },
	addOfflineAccountHeader: {
		id: 'accounts-card.add-offline-account-header',
		defaultMessage: 'Add new offline account',
	},
	enterPlayerName: {
		id: 'accounts-card.enter-player-name',
		defaultMessage: 'Enter your player name',
	},
	playerNamePlaceholder: {
		id: 'accounts-card.player-name-placeholder',
		defaultMessage: 'Your player name here...',
	},
	errorAuthElyByHeader: {
		id: 'accounts-card.error-auth-elyby-header',
		defaultMessage: 'Error while proceeding authentication event with Ely.by',
	},
	errorWhileLoggingIn: {
		id: 'accounts-card.error-while-logging-in',
		defaultMessage: 'An error occurred while logging in.',
	},
	tryAgain: { id: 'action.try-again', defaultMessage: 'Try again' },
	errorInputElyByHeader: {
		id: 'accounts-card.error-input-elyby-header',
		defaultMessage: 'Error while proceeding input event with Ely.by',
	},
	errorAddingElyByAccount: {
		id: 'accounts-card.error-adding-elyby-account',
		defaultMessage:
			'An error occurred while adding the Ely.by account. Please follow the instructions below.',
	},
	checkPlayerNameOrEmail: {
		id: 'accounts-card.check-player-name-or-email',
		defaultMessage: 'Check that you have entered the correct player name or email.',
	},
	checkPassword: {
		id: 'accounts-card.check-password',
		defaultMessage: 'Check that you have entered the correct password.',
	},
	errorInputOfflineHeader: {
		id: 'accounts-card.error-input-offline-header',
		defaultMessage: 'Error while proceeding input event with offline account',
	},
	errorAddingOfflineAccount: {
		id: 'accounts-card.error-adding-offline-account',
		defaultMessage:
			'An error occurred while adding the offline account. Please follow the instructions below.',
	},
	checkPlayerName: {
		id: 'accounts-card.check-player-name',
		defaultMessage: 'Check that you have entered the correct player name.',
	},
	playerNameLengthRule: {
		id: 'accounts-card.player-name-length-rule',
		defaultMessage:
			'Player name must be at least {min} characters long and no more than {max} characters.',
	},
	unexpectedErrorHeader: {
		id: 'accounts-card.unexpected-error-header',
		defaultMessage: 'Unexpected error occurred',
	},
	unexpectedErrorBody: {
		id: 'accounts-card.unexpected-error-body',
		defaultMessage: 'An unexpected error has occurred. Please try again later.',
	},
})

defineProps({
	mode: {
		type: String,
		required: true,
		default: 'normal',
	},
})

const emit = defineEmits(['change'])

const accounts = ref({})
const microsoftLoginDisabled = ref(false)
const elybyLoginDisabled = ref(false)
const defaultUser = ref()

// [AR] • Feature
const clientToken = 'revoria'
const addOfflineModal = ref(null)
const addElybyModal = ref(null)
const requestElybyTwoFactorCodeModal = ref(null)
const authenticationElybyErrorModal = ref(null)
const inputElybyErrorModal = ref(null)
const inputErrorModal = ref(null)
const exceptionErrorModal = ref(null)
const offlinePlayerName = ref('')
const elybyLogin = ref('')
const elybyPassword = ref('')
const elybyTwoFactorCode = ref('')
const minOfflinePlayerNameLength = 2
const maxOfflinePlayerNameLength = 20

// [AR] • Feature
function getAccountType(account) {
	switch (account.account_type) {
		case 'microsoft':
			return License
		case 'pirate':
			return Offline
		case 'elyby':
			return Elyby
	}
}

// [AR] • Feature
function showOfflineLoginModal() {
	addOfflineModal.value?.show()
}

// [AR] • Feature
function showElybyLoginModal() {
	addElybyModal.value?.show()
}

// [AR] • Feature
function retryAddOfflineProfile() {
	inputErrorModal.value?.hide()
	clearOfflineFields()
	showOfflineLoginModal()
}

// [AR] • Feature
function retryAddElybyProfile() {
	authenticationElybyErrorModal.value?.hide()
	inputElybyErrorModal.value?.hide()
	clearElybyFields()
	showElybyLoginModal()
}

// [AR] • Feature
function clearElybyFields() {
	elybyLogin.value = ''
	elybyPassword.value = ''
	elybyTwoFactorCode.value = ''
}

// [AR] • Feature
function clearOfflineFields() {
	offlinePlayerName.value = ''
}

// [AR] • Feature
async function addOfflineProfile() {
	const name = offlinePlayerName.value.trim()
	const isValidName =
		name.length >= minOfflinePlayerNameLength && name.length <= maxOfflinePlayerNameLength

	if (!isValidName) {
		addOfflineModal.value?.hide()
		inputErrorModal.value?.show()
		clearOfflineFields()
		return
	}

	try {
		const result = await offline_login(name)

		addOfflineModal.value?.hide()

		if (result) {
			await setAccount(result)
			await refreshValues()
		} else {
			exceptionErrorModal.value?.show()
		}
	} catch (error) {
		handleError(error)
		exceptionErrorModal.value?.show()
	} finally {
		clearOfflineFields()
	}
}

// [AR] • Feature
async function addElybyProfile() {
	if (!elybyLogin.value || !elybyPassword.value) {
		addElybyModal.value?.hide()
		inputElybyErrorModal.value?.show()
		clearElybyFields()
		return
	}
	elybyLoginDisabled.value = true

	const login = elybyLogin.value.trim()
	let password = elybyPassword.value.trim()
	const twoFactorCode = elybyTwoFactorCode.value.trim()
	if (password && twoFactorCode) {
		password = `${password}:${twoFactorCode}`
	}

	try {
		const raw_result = await elyby_auth_authenticate(login, password, clientToken)

		const json_data = JSON.parse(raw_result)

		console.log(json_data?.error)
		console.log(json_data?.errorMessage)

		if (!json_data.accessToken) {
			if (
				json_data.error === 'ForbiddenOperationException' &&
				json_data.errorMessage?.includes('two factor')
			) {
				requestElybyTwoFactorCodeModal.value?.show()
				return
			}

			addElybyModal.value?.hide()
			requestElybyTwoFactorCodeModal.value?.hide()
			authenticationElybyErrorModal.value?.show()
			return
		}

		const accessToken = json_data.accessToken
		const selectedProfileId = convertRawStringToUUIDv4(json_data.selectedProfile.id)
		const selectedProfileName = json_data.selectedProfile.name

		const result = await elyby_login(selectedProfileId, selectedProfileName, accessToken)

		addElybyModal.value?.hide()
		requestElybyTwoFactorCodeModal.value?.hide()

		clearElybyFields()

		await setAccount(result)
		await refreshValues()
	} catch (err) {
		handleError(err)
		exceptionErrorModal.value?.show()
	} finally {
		elybyLoginDisabled.value = false
	}
}

// [AR] • Feature
function convertRawStringToUUIDv4(rawId) {
	if (rawId.length !== 32) {
		console.warn('Invalid UUID string:', rawId)
		return rawId
	}
	return `${rawId.slice(0, 8)}-${rawId.slice(8, 12)}-${rawId.slice(12, 16)}-${rawId.slice(16, 20)}-${rawId.slice(20)}`
}

const equippedSkin = ref(null)
const headUrlCache = ref(new Map())

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	accounts.value = await users().catch(handleError)

	try {
		const skins = await get_available_skins()
		equippedSkin.value = skins.find((skin) => skin.is_equipped)

		if (equippedSkin.value) {
			try {
				const headUrl = await getPlayerHeadUrl(equippedSkin.value)
				headUrlCache.value.set(equippedSkin.value.texture_key, headUrl)
			} catch (error) {
				console.warn('Failed to get head render for equipped skin:', error)
			}
		}
	} catch {
		equippedSkin.value = null
	}
}

function setLoginDisabled(value) {
	microsoftLoginDisabled.value = value
}

defineExpose({
	refreshValues,
	setLoginDisabled,
	microsoftLoginDisabled,
})
await refreshValues()

const displayAccounts = computed(() =>
	accounts.value.filter((account) => defaultUser.value !== account.profile.id),
)

const avatarUrl = computed(() => {
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
		return `https://mc-heads.net/avatar/${equippedSkin.value.texture_key}/128`
	}
	if (selectedAccount.value?.profile?.id) {
		return `https://mc-heads.net/avatar/${selectedAccount.value.profile.id}/128`
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account) {
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = headUrlCache.value.get(equippedSkin.value.texture_key)
		if (cachedUrl) {
			return cachedUrl
		}
	}
	return `https://mc-heads.net/avatar/${account.profile.id}/128`
}

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

async function setAccount(account) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	emit('change')
}

async function login() {
	microsoftLoginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
		await refreshValues()
	}

	trackEvent('AccountLogIn')
	microsoftLoginDisabled.value = false
}

const logout = async (id) => {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
		await refreshValues()
	} else {
		emit('change')
	}
	trackEvent('AccountLogOut')
}

const showCard = ref(false)
const card = ref(null)
const button = ref(null)
const handleClickOutside = (event) => {
	const elements = document.elementsFromPoint(event.clientX, event.clientY)
	if (
		card.value &&
		card.value.$el !== event.target &&
		!elements.includes(card.value.$el) &&
		!button.value.contains(event.target)
	) {
		toggleMenu(false)
	}
}

function toggleMenu(override = true) {
	if (showCard.value || !override) {
		showCard.value = false
	} else {
		showCard.value = true
	}
}

const unlisten = await process_listener(async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

onMounted(() => {
	window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
	window.removeEventListener('click', handleClickOutside)
})

onUnmounted(() => {
	unlisten()
})
</script>

<style scoped lang="scss">
.selected {
	background: var(--color-brand-highlight);
	border-radius: var(--radius-lg);
	color: var(--color-contrast);
	gap: 1rem;
}

.login-section {
	background: var(--color-bg);
	border-radius: var(--radius-lg);
	gap: 1rem;
}

.vector-icon {
	width: 12px;
	height: 12px;
	display: inline-block;
	vertical-align: middle;

	:deep(svg) {
		display: block;
		width: 12px;
		height: 12px;
	}
}

.dropdown-icon {
	width: 1.25rem;
	height: 1.25rem;
	display: block;
}

.account-title {
	display: inline-flex;
	align-items: center;
	gap: 0.5rem;
	line-height: 1.1;
}

.account {
	width: max-content;
	display: flex;
	align-items: center;
	line-height: 1.1;
	text-align: left;
	padding: 0.5rem 1rem;

	h4,
	p {
		margin: 0;
	}
}

.account-card {
	position: fixed;
	display: flex;
	flex-direction: column;
	margin-top: 0.5rem;
	right: 2rem;
	z-index: 11;
	gap: 0.5rem;
	padding: 1rem;
	border: 1px solid var(--color-divider);
	width: max-content;
	user-select: none;
	-ms-user-select: none;
	-webkit-user-select: none;
	max-height: calc(100vh - 300px);
	overflow-y: auto;

	&::-webkit-scrollbar-track {
		border-top-right-radius: 1rem;
		border-bottom-right-radius: 1rem;
	}

	&::-webkit-scrollbar {
		border-top-right-radius: 1rem;
		border-bottom-right-radius: 1rem;
	}

	&.hidden {
		display: none;
	}

	&.expanded {
		left: 13.5rem;
	}

	&.isolated {
		position: relative;
		left: 0;
		top: 0;
	}
}

.accounts-title {
	font-size: 1.2rem;
	font-weight: bolder;
}

.centered {
	display: flex;
	gap: 1rem;
	margin: auto;
}

.account-group {
	width: 100%;
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
}

.option {
	width: calc(100% - 2.25rem);
	background: var(--color-raised-bg);
	color: var(--color-base);
	box-shadow: none;

	img {
		margin-right: 0.5rem;
	}
}

.icon {
	--size: 1.5rem !important;
}

.account-row {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
	vertical-align: center;
	justify-content: space-between;
	padding-right: 1rem;
}

.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 0.25s ease,
		translate 0.25s ease,
		scale 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	translate: 0 -2rem;
	scale: 0.9;
}

.avatar-button {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	color: var(--color-base);
	background-color: var(--color-button-bg);
	border-radius: var(--radius-md);
	width: 100%;
	padding: 0.5rem 0.75rem;
	text-align: left;

	&.expanded {
		border: 1px solid var(--color-divider);
		padding: 1rem;
	}
}

.avatar-text {
	margin: auto 0 auto 0.25rem;
	display: flex;
	flex-direction: column;
}

.text {
	width: 6rem;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.accounts-text {
	display: flex;
	align-items: center;
	gap: 0.25rem;
	margin: 0;
}

.qr-code {
	background-color: white !important;
	border-radius: var(--radius-md);
}

.modal-body {
	display: flex;
	flex-direction: row;
	gap: var(--gap-lg);
	align-items: center;
	padding: var(--gap-xl);

	.modal-text {
		display: flex;
		flex-direction: column;
		gap: var(--gap-sm);
		width: 100%;

		h2,
		p {
			margin: 0;
		}

		.code-text {
			display: flex;
			flex-direction: row;
			gap: var(--gap-xs);
			align-items: center;

			.code {
				background-color: var(--color-bg);
				border-radius: var(--radius-md);
				border: solid 1px var(--color-button-bg);
				font-family: var(--mono-font);
				letter-spacing: var(--gap-md);
				color: var(--color-contrast);
				font-size: 2rem;
				font-weight: bold;
				padding: var(--gap-sm) 0 var(--gap-sm) var(--gap-md);
			}

			.btn {
				width: 2.5rem;
				height: 2.5rem;
			}
		}
	}
}

.button-row {
	display: flex;
	flex-direction: row;
}

.modal {
	position: absolute;
}

.code {
	color: var(--color-brand);
	padding: 0.05rem 0.1rem;
	// row not column
	display: flex;

	.card {
		background: var(--color-base);
		color: var(--color-contrast);
		padding: 0.5rem 1rem;
	}
}
</style>
