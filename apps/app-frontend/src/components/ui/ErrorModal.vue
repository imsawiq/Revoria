<script setup>
import {
	CheckIcon,
	CopyIcon,
	DropdownIcon,
	HammerIcon,
	LogInIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import { ButtonStyled, Collapsible, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref } from 'vue'

import { ChatIcon } from '@/assets/icons'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { install } from '@/helpers/profile.js'
import { cancel_directory_change } from '@/helpers/settings.ts'
import { handleSevereError } from '@/store/error.js'

// [AR] Imports
import { applyMigrationFix, restartApp } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	titleErrorOccurred: {
		id: 'error-modal.title.error-occurred',
		defaultMessage: 'An error occurred',
	},
	titleUnableSignIn: {
		id: 'error-modal.title.unable-sign-in',
		defaultMessage: 'Unable to sign in to Minecraft',
	},
	titleSignIn: { id: 'error-modal.title.sign-in', defaultMessage: 'Sign in to Minecraft' },
	titleChangeDirectory: {
		id: 'error-modal.title.change-directory',
		defaultMessage: 'Could not change app directory',
	},
	titleNoLoader: { id: 'error-modal.title.no-loader', defaultMessage: 'No loader selected' },
	titleStateInit: {
		id: 'error-modal.title.state-init',
		defaultMessage: 'Error initializing Revoria',
	},
	noErrorMessage: { id: 'error-modal.no-error-message', defaultMessage: 'No error message.' },
	networkIssues: { id: 'error-modal.network-issues', defaultMessage: 'Network issues' },
	minecraftAuthNetworkTextStart: {
		id: 'error-modal.minecraft-auth.network-text-start',
		defaultMessage:
			"It looks like there were issues with Revoria connecting to Microsoft's servers. This is often the result of a poor connection, so we recommend trying again to see if it works. If issues continue to persist, follow the steps in",
	},
	ourSupportArticle: {
		id: 'error-modal.our-support-article',
		defaultMessage: 'our support article',
	},
	minecraftAuthNetworkTextEnd: {
		id: 'error-modal.minecraft-auth.network-text-end',
		defaultMessage: 'to troubleshoot.',
	},
	minecraftAuthHostsTextStart: {
		id: 'error-modal.minecraft-auth.hosts-text-start',
		defaultMessage:
			'The Revoria launcher tried to connect to Microsoft / Xbox / Minecraft services, but the remote server rejected the connection. This may indicate that these services are blocked by the hosts file. Please visit',
	},
	minecraftAuthHostsTextEnd: {
		id: 'error-modal.minecraft-auth.hosts-text-end',
		defaultMessage: 'for steps on how to fix the issue.',
	},
	tryAnotherMicrosoftAccount: {
		id: 'error-modal.try-another-microsoft-account',
		defaultMessage: 'Try another Microsoft account',
	},
	minecraftAuthTryAnotherText: {
		id: 'error-modal.minecraft-auth.try-another-text',
		defaultMessage:
			"Double check you've signed in with the right account. You may own Minecraft on a different Microsoft account.",
	},
	tryAnotherAccount: {
		id: 'error-modal.try-another-account',
		defaultMessage: 'Try another account',
	},
	minecraftAuthGamePassHeading: {
		id: 'error-modal.minecraft-auth.game-pass-heading',
		defaultMessage: 'Using PC Game Pass, coming from Bedrock, or just bought the game?',
	},
	minecraftAuthGamePassTextStart: {
		id: 'error-modal.minecraft-auth.game-pass-text-start',
		defaultMessage: 'Try signing in with the',
	},
	officialMinecraftLauncher: {
		id: 'error-modal.official-minecraft-launcher',
		defaultMessage: 'official Minecraft Launcher',
	},
	minecraftAuthGamePassTextEnd: {
		id: 'error-modal.minecraft-auth.game-pass-text-end',
		defaultMessage: "first. Once you're done, come back here and sign in!",
	},
	trySigningInAgain: {
		id: 'error-modal.try-signing-in-again',
		defaultMessage: 'Try signing in again',
	},
	changeDirectoryPermissions: {
		id: 'error-modal.change-directory-permissions',
		defaultMessage: 'Change directory permissions',
	},
	directoryReadOnlyText: {
		id: 'error-modal.directory-read-only-text',
		defaultMessage:
			'It looks like Revoria is unable to write to the directory you selected. Please adjust the permissions of the directory and try again or cancel the directory change.',
	},
	notEnoughSpace: { id: 'error-modal.not-enough-space', defaultMessage: 'Not enough space' },
	notEnoughSpaceText: {
		id: 'error-modal.not-enough-space-text',
		defaultMessage:
			'It looks like there is not enough space on the disk containing the directory you selected. Please free up some space and try again or cancel the directory change.',
	},
	directoryMoveGenericText: {
		id: 'error-modal.directory-move-generic-text',
		defaultMessage:
			'Revoria is unable to migrate to the new directory you selected. Please contact support for help or cancel the directory change.',
	},
	retryDirectoryChange: {
		id: 'error-modal.retry-directory-change',
		defaultMessage: 'Retry directory change',
	},
	cancelDirectoryChange: {
		id: 'error-modal.cancel-directory-change',
		defaultMessage: 'Cancel directory change',
	},
	minecraftSignInTextStart: {
		id: 'error-modal.minecraft-sign-in-text-start',
		defaultMessage:
			"To play this instance, you must sign in through Microsoft below. If you don't have a Minecraft account, you can purchase the game on the",
	},
	minecraftWebsite: { id: 'error-modal.minecraft-website', defaultMessage: 'Minecraft website' },
	signInToMinecraft: {
		id: 'error-modal.sign-in-to-minecraft',
		defaultMessage: 'Sign in to Minecraft',
	},
	stateInitText: {
		id: 'error-modal.state-init-text',
		defaultMessage:
			'Revoria failed to load correctly. This may be because of a corrupted file, or because the app is missing crucial files.',
	},
	stateInitFixWays: {
		id: 'error-modal.state-init-fix-ways',
		defaultMessage: 'You may be able to fix it through one of the following ways:',
	},
	stateInitFixInternet: {
		id: 'error-modal.state-init-fix-internet',
		defaultMessage: 'Ensuring you are connected to the internet, then try restarting the app.',
	},
	stateInitFixRedownload: {
		id: 'error-modal.state-init-fix-redownload',
		defaultMessage: 'Redownloading the app.',
	},
	noLoaderText: {
		id: 'error-modal.no-loader-text',
		defaultMessage: 'Revoria failed to find the loader version for this instance.',
	},
	noLoaderRepairText: {
		id: 'error-modal.no-loader-repair-text',
		defaultMessage:
			'To resolve this, you need to repair the instance. Click the button below to do so.',
	},
	repairInstance: { id: 'error-modal.repair-instance', defaultMessage: 'Repair instance' },
	debugHelpTextStart: {
		id: 'error-modal.debug-help-text-start',
		defaultMessage: 'If nothing is working and you need help, visit',
	},
	debugHelpSupportPage: {
		id: 'error-modal.debug-help-support-page',
		defaultMessage: 'our support page',
	},
	debugHelpTextEnd: {
		id: 'error-modal.debug-help-text-end',
		defaultMessage:
			'and start a chat using the widget in the bottom right and we will be more than happy to assist! Make sure to provide the following debug information to the agent:',
	},
	getSupport: { id: 'error-modal.get-support', defaultMessage: 'Get support' },
	close: { id: 'error-modal.close', defaultMessage: 'Close' },
	copied: { id: 'error-modal.copied', defaultMessage: 'Copied!' },
	copyDebugInfo: { id: 'error-modal.copy-debug-info', defaultMessage: 'Copy debug info' },
	debugInformation: { id: 'error-modal.debug-information', defaultMessage: 'Debug information:' },
	migrationHeader: {
		id: 'error-modal.migration.header',
		defaultMessage: '⚠️ Migration Issue • Important Notice',
	},
	migrationText1: {
		id: 'error-modal.migration.text-1',
		defaultMessage:
			"We've detected a problem with our database migration system caused by inconsistent line endings between operating systems (Windows vs. macOS/Linux). This may affect app stability.",
	},
	migrationText2: {
		id: 'error-modal.migration.text-2',
		defaultMessage:
			'What’s happening? Our migration validator misreads modified migrations when line endings differ (CRLF ↔ LF), which can make the app unusable.',
	},
	migrationText3: {
		id: 'error-modal.migration.text-3',
		defaultMessage:
			'Why? Git’s automatic line-ending conversions and OS differences can cause these inconsistencies during builds.',
	},
	migrationText4: {
		id: 'error-modal.migration.text-4',
		defaultMessage:
			'What’s next? We’re working on a permanent fix. In the meantime, you can apply one of the quick fixes below depending on your system.',
	},
	migrationNeedFixNow: {
		id: 'error-modal.migration.need-fix-now',
		defaultMessage: 'Do I need to apply a fix now?',
	},
	migrationHelpText: {
		id: 'error-modal.migration.help-text',
		defaultMessage:
			'If you are encountering an error while applying migrations, such as "Error while applying migrations: migration XXXXXXXXXX was previously applied but has been modified", or a similar issue with migration, the following actions might help:',
	},
	migrationBackupText: {
		id: 'error-modal.migration.backup-text',
		defaultMessage:
			'If none of the above steps help, you can try saving a copy of the file app.db to a safe location, such as %appdata%\\Roaming\\Revoria on Windows or ~/Library/Application Support/Revoria on macOS, then deleting the original file and letting the app re-create the database file. Note that this may cause data loss inside the app, so make sure to back up your launcher data before applying these fixes.',
	},
	migrationFixUnixTitle: {
		id: 'error-modal.migration.fix-unix-title',
		defaultMessage: 'Convert all line endings in migration files to LF (Unix-style: \\n)',
	},
	migrationFixWindowsTitle: {
		id: 'error-modal.migration.fix-windows-title',
		defaultMessage: 'Convert all line endings in migration files to CRLF (Windows-style: \\r\\n)',
	},
	migrationFixUnixButton: {
		id: 'error-modal.migration.fix-unix-button',
		defaultMessage: 'Apply fix for Unix like systems (Debian, Ubuntu, macOS and others)',
	},
	migrationFixWindowsButton: {
		id: 'error-modal.migration.fix-windows-button',
		defaultMessage: 'Apply fix for Windows',
	},
	migrationReportHeader: {
		id: 'error-modal.migration.report-header',
		defaultMessage: '💡 Migration fix report',
	},
	migrationFixSuccess: {
		id: 'error-modal.migration.fix-success',
		defaultMessage:
			'✅ The migration fix has been applied successfully. Please restart the launcher and try to log in to the game :)',
	},
	migrationFixFailed: {
		id: 'error-modal.migration.fix-failed',
		defaultMessage: '❌ The migration fix failed or had no effect.',
	},
	migrationTryOtherFix: {
		id: 'error-modal.migration.try-other-fix',
		defaultMessage: 'If the problem persists, please try the other fix.',
	},
})

const errorModal = ref()
const error = ref()
const closable = ref(true)
const errorCollapsed = ref(false)
const migrationFixSuccess = ref(null) // null | true | false
const migrationFixCallbackModel = ref()

const title = ref(formatMessage(messages.titleErrorOccurred))
const errorType = ref('unknown')
const supportLink = ref('https://support.modrinth.com')
const metadata = ref({})

defineExpose({
	async show(errorVal, context, canClose = true, source = null) {
		closable.value = canClose

		if (errorVal.message && errorVal.message.includes('Minecraft authentication error:')) {
			title.value = formatMessage(messages.titleUnableSignIn)
			errorType.value = 'minecraft_auth'
			supportLink.value =
				'https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues'

			if (
				errorVal.message.includes('existing connection was forcibly closed') ||
				errorVal.message.includes('error sending request for url')
			) {
				metadata.value.network = true
			}
			if (errorVal.message.includes('because the target machine actively refused it')) {
				metadata.value.hostsFile = true
			}
		} else if (errorVal.message && errorVal.message.includes('User is not logged in')) {
			title.value = formatMessage(messages.titleSignIn)
			errorType.value = 'minecraft_sign_in'
			supportLink.value = 'https://support.modrinth.com'
		} else if (errorVal.message && errorVal.message.includes('Move directory error:')) {
			title.value = formatMessage(messages.titleChangeDirectory)
			errorType.value = 'directory_move'
			supportLink.value = 'https://support.modrinth.com'

			if (errorVal.message.includes('directory is not writeable')) {
				metadata.value.readOnly = true
			}

			if (errorVal.message.includes('Not enough space')) {
				metadata.value.notEnoughSpace = true
			}
		} else if (errorVal.message && errorVal.message.includes('No loader version selected for')) {
			title.value = formatMessage(messages.titleNoLoader)
			errorType.value = 'no_loader_version'
			supportLink.value = 'https://support.modrinth.com'
			metadata.value.profilePath = context.profilePath
		} else if (source === 'state_init') {
			title.value = formatMessage(messages.titleStateInit)
			errorType.value = 'state_init'
			supportLink.value = 'https://support.modrinth.com'
		} else {
			title.value = formatMessage(messages.titleErrorOccurred)
			errorType.value = 'unknown'
			supportLink.value = 'https://support.modrinth.com'
			metadata.value = {}
		}

		error.value = errorVal
		errorModal.value.show()
	},
})

const loadingMinecraft = ref(false)
async function loginMinecraft() {
	try {
		loadingMinecraft.value = true
		const loggedIn = await login_flow()

		if (loggedIn) {
			await set_default_user(loggedIn.profile.id).catch(handleError)
		}

		await trackEvent('AccountLogIn', { source: 'ErrorModal' })
		loadingMinecraft.value = false
		errorModal.value.hide()
	} catch (err) {
		loadingMinecraft.value = false
		handleSevereError(err)
	}
}

async function cancelDirectoryChange() {
	try {
		await cancel_directory_change()
		window.location.reload()
	} catch (err) {
		handleError(err)
	}
}

function retryDirectoryChange() {
	window.location.reload()
}

const loadingRepair = ref(false)
async function repairInstance() {
	loadingRepair.value = true
	try {
		await install(metadata.value.profilePath, false)
		errorModal.value.hide()
	} catch (err) {
		handleSevereError(err)
	}
	loadingRepair.value = false
}

const hasDebugInfo = computed(
	() =>
		errorType.value === 'directory_move' ||
		errorType.value === 'minecraft_auth' ||
		errorType.value === 'state_init' ||
		errorType.value === 'no_loader_version',
)

const debugInfo = computed(
	() => error.value.message ?? error.value ?? formatMessage(messages.noErrorMessage),
)

const copied = ref(false)

async function copyToClipboard(text) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}

async function onApplyMigrationFix(eol) {
	console.log(`[AR] • Attempting to apply migration ${eol.toUpperCase()} fix`)
	try {
		const result = await applyMigrationFix(eol)
		migrationFixSuccess.value = result === true
		console.log(`[AR] • Successfully applied migration ${eol.toUpperCase()} fix`, result)
	} catch (err) {
		console.error(`[AR] • Failed to apply migration fix:`, err)
		migrationFixSuccess.value = false
	} finally {
		migrationFixCallbackModel.value?.show?.()
		if (migrationFixSuccess.value === true) {
			setTimeout(async () => {
				await restartApp()
			}, 3000)
		}
	}
}
</script>

<template>
	<ModalWrapper ref="errorModal" :header="title" :closable="closable">
		<div class="modal-body">
			<div class="markdown-body">
				<template v-if="errorType === 'minecraft_auth'">
					<template v-if="metadata.network">
						<h3>{{ formatMessage(messages.networkIssues) }}</h3>
						<p>
							{{ formatMessage(messages.minecraftAuthNetworkTextStart) }}
							<a
								href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_e71a5f805f"
							>
								{{ formatMessage(messages.ourSupportArticle) }}
							</a>
							{{ formatMessage(messages.minecraftAuthNetworkTextEnd) }}
						</p>
					</template>
					<template v-else-if="metadata.hostsFile">
						<h3>{{ formatMessage(messages.networkIssues) }}</h3>
						<p>
							{{ formatMessage(messages.minecraftAuthHostsTextStart) }}
							<a
								href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_d694a29256"
							>
								{{ formatMessage(messages.ourSupportArticle) }}
							</a>
							{{ formatMessage(messages.minecraftAuthHostsTextEnd) }}
						</p>
					</template>
					<template v-else>
						<h3>{{ formatMessage(messages.tryAnotherMicrosoftAccount) }}</h3>
						<p>
							{{ formatMessage(messages.minecraftAuthTryAnotherText) }}
						</p>
						<div class="cta-button">
							<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
								<LogInIcon /> {{ formatMessage(messages.tryAnotherAccount) }}
							</button>
						</div>
						<h3>{{ formatMessage(messages.minecraftAuthGamePassHeading) }}</h3>
						<p>
							{{ formatMessage(messages.minecraftAuthGamePassTextStart) }}
							<a href="https://www.minecraft.net/en-us/download">{{
								formatMessage(messages.officialMinecraftLauncher)
							}}</a>
							{{ formatMessage(messages.minecraftAuthGamePassTextEnd) }}
						</p>
					</template>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
							<LogInIcon /> {{ formatMessage(messages.trySigningInAgain) }}
						</button>
					</div>
				</template>
				<template v-if="errorType === 'directory_move'">
					<template v-if="metadata.readOnly">
						<h3>{{ formatMessage(messages.changeDirectoryPermissions) }}</h3>
						<p>{{ formatMessage(messages.directoryReadOnlyText) }}</p>
					</template>
					<template v-else-if="metadata.notEnoughSpace">
						<h3>{{ formatMessage(messages.notEnoughSpace) }}</h3>
						<p>{{ formatMessage(messages.notEnoughSpaceText) }}</p>
					</template>
					<template v-else>
						<p>{{ formatMessage(messages.directoryMoveGenericText) }}</p>
					</template>

					<div class="cta-button">
						<button class="btn" @click="retryDirectoryChange">
							<UpdatedIcon /> {{ formatMessage(messages.retryDirectoryChange) }}
						</button>
						<button class="btn btn-danger" @click="cancelDirectoryChange">
							<XIcon /> {{ formatMessage(messages.cancelDirectoryChange) }}
						</button>
					</div>
				</template>
				<div v-else-if="errorType === 'minecraft_sign_in'">
					<p>
						{{ formatMessage(messages.minecraftSignInTextStart) }}
						<a href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc">{{
							formatMessage(messages.minecraftWebsite)
						}}</a
						>.
					</p>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
							<LogInIcon /> {{ formatMessage(messages.signInToMinecraft) }}
						</button>
					</div>
				</div>
				<template v-else-if="errorType === 'state_init'">
					<p>{{ formatMessage(messages.stateInitText) }}</p>
					<p>{{ formatMessage(messages.stateInitFixWays) }}</p>
					<ul>
						<li>{{ formatMessage(messages.stateInitFixInternet) }}</li>
						<li>{{ formatMessage(messages.stateInitFixRedownload) }}</li>
					</ul>
				</template>
				<template v-else-if="errorType === 'no_loader_version'">
					<p>{{ formatMessage(messages.noLoaderText) }}</p>
					<p>{{ formatMessage(messages.noLoaderRepairText) }}</p>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingRepair" @click="repairInstance">
							<HammerIcon /> {{ formatMessage(messages.repairInstance) }}
						</button>
					</div>
				</template>
				<template v-else>
					{{ debugInfo }}
				</template>
				<template v-if="hasDebugInfo">
					<hr />
					<p>
						{{ formatMessage(messages.debugHelpTextStart) }}
						<a :href="supportLink">{{ formatMessage(messages.debugHelpSupportPage) }}</a>
						{{ formatMessage(messages.debugHelpTextEnd) }}
					</p>
				</template>
			</div>
			<div class="flex items-center gap-2">
				<ButtonStyled>
					<a :href="supportLink" @click="errorModal.hide()"
						><ChatIcon /> {{ formatMessage(messages.getSupport) }}</a
					>
				</ButtonStyled>
				<ButtonStyled v-if="closable">
					<button @click="errorModal.hide()"><XIcon /> {{ formatMessage(messages.close) }}</button>
				</ButtonStyled>
				<ButtonStyled v-if="hasDebugInfo">
					<button :disabled="copied" @click="copyToClipboard(debugInfo)">
						<template v-if="copied">
							<CheckIcon class="text-green" /> {{ formatMessage(messages.copied) }}
						</template>
						<template v-else> <CopyIcon /> {{ formatMessage(messages.copyDebugInfo) }} </template>
					</button>
				</ButtonStyled>
			</div>
			<template v-if="hasDebugInfo">
				<div class="bg-button-bg rounded-xl mt-2 overflow-clip">
					<button
						class="flex items-center justify-between w-full bg-transparent border-0 px-4 py-3 cursor-pointer"
						@click="errorCollapsed = !errorCollapsed"
					>
						<span class="text-contrast font-extrabold m-0">{{
							formatMessage(messages.debugInformation)
						}}</span>
						<DropdownIcon
							class="h-5 w-5 text-secondary transition-transform"
							:class="{ 'rotate-180': !errorCollapsed }"
						/>
					</button>
					<Collapsible :collapsed="errorCollapsed">
						<pre
							class="m-0 px-4 py-3 bg-bg rounded-none whitespace-pre-wrap break-words overflow-x-auto max-w-full"
							>{{ debugInfo }}</pre
						>
					</Collapsible>
				</div>
				<template v-if="errorType === 'state_init'">
					<h2>{{ formatMessage(messages.migrationHeader) }}</h2>
					<p>{{ formatMessage(messages.migrationText1) }}</p>
					<p>
						<strong>{{ formatMessage(messages.migrationText2) }}</strong>
					</p>
					<p>
						<strong>{{ formatMessage(messages.migrationText3) }}</strong>
					</p>
					<p>
						<strong>{{ formatMessage(messages.migrationText4) }}</strong>
					</p>
					<h3>{{ formatMessage(messages.migrationNeedFixNow) }}</h3>
					<div>
						<p class="notice__text">
							{{ formatMessage(messages.migrationHelpText) }}
						</p>
						<p>{{ formatMessage(messages.migrationBackupText) }}</p>
					</div>
					<div class="flex justify-between">
						<ol class="flex flex-col gap-3">
							<li>
								<ButtonStyled class="neon-button neon">
									<button
										:title="formatMessage(messages.migrationFixUnixTitle)"
										@click="onApplyMigrationFix('lf')"
									>
										{{ formatMessage(messages.migrationFixUnixButton) }}
									</button>
								</ButtonStyled>
							</li>
							<li>
								<ButtonStyled class="neon-button neon">
									<button
										:title="formatMessage(messages.migrationFixWindowsTitle)"
										@click="onApplyMigrationFix('crlf')"
									>
										{{ formatMessage(messages.migrationFixWindowsButton) }}
									</button>
								</ButtonStyled>
							</li>
						</ol>
					</div>
				</template>
			</template>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="migrationFixCallbackModel"
		:header="formatMessage(messages.migrationReportHeader)"
		:closable="closable"
	>
		<div class="modal-body">
			<h2 class="text-lg font-bold text-contrast space-y-2">
				<template v-if="migrationFixSuccess === true">
					<p class="flex items-center gap-2 neon-text">
						{{ formatMessage(messages.migrationFixSuccess) }}
					</p>
					<p class="mt-2 text-sm neon-text">
						{{ formatMessage(messages.migrationTryOtherFix) }}
					</p>
				</template>

				<template v-else-if="migrationFixSuccess === false">
					<p class="flex items-center gap-2 neon-text">
						{{ formatMessage(messages.migrationFixFailed) }}
					</p>
					<p class="mt-2 text-sm neon-text">
						{{ formatMessage(messages.migrationTryOtherFix) }}
					</p>
				</template>
			</h2>
		</div>
	</ModalWrapper>
</template>

<style>
.light-mode {
	--color-orange-bg: rgba(255, 163, 71, 0.2);
}

.dark-mode,
.oled-mode {
	--color-orange-bg: rgba(224, 131, 37, 0.2);
}
</style>

<style scoped lang="scss">
@import '../../../../../packages/assets/styles/neon-button.scss';
@import '../../../../../packages/assets/styles/neon-text.scss';

code {
	background: linear-gradient(90deg, #005eff, #00cfff);
	background-clip: text;
	-webkit-background-clip: text;
	color: transparent;
}

.cta-button {
	display: flex;
	align-items: center;
	justify-content: center;
	padding: 0.5rem;
	gap: 0.5rem;
}

.warning-banner {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: var(--gap-lg);
	background-color: var(--color-orange-bg);
	border: 2px solid var(--color-orange);
	border-radius: var(--radius-md);
	margin-bottom: 1rem;
}

.warning-banner__title {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	font-weight: 700;

	svg {
		color: var(--color-orange);
		height: 1.5rem;
		width: 1.5rem;
	}
}

.modal-body {
	display: flex;
	flex-direction: column;
	gap: var(--gap-md);
}

.markdown-body {
	overflow: auto;
}
</style>
