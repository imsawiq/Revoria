<template>
	<div>
		<Teleport v-if="data" to="#sidebar-teleport-target">
			<div>
				<template v-if="!isCurseForge">
					<ProjectSidebarCompatibility
						:project="data"
						:tags="{ loaders: allLoaders ?? [], gameVersions: allGameVersions ?? [] }"
						class="project-sidebar-section"
					/>
					<ProjectSidebarLinks link-target="_blank" :project="data" class="project-sidebar-section" />
				</template>
				<template v-else>
					<div class="project-sidebar-section">
						<h3 class="m-0 text-sm font-extrabold uppercase tracking-wide text-secondary">
							{{ formatMessage(messages.curseForgeProject) }}
						</h3>
						<div v-if="data.game_versions?.length" class="flex flex-wrap gap-1">
							<span
								v-for="v in data.game_versions.slice(0, 6)"
								:key="v"
								class="rounded-md bg-[--color-button-bg] px-2 py-0.5 text-xs font-medium"
							>
								{{ v }}
							</span>
							<span v-if="data.game_versions.length > 6" class="text-xs text-secondary">
								+{{ data.game_versions.length - 6 }} {{ formatMessage(messages.more) }}
							</span>
						</div>
						<a
							v-if="openInBrowserLink"
							:href="openInBrowserLink"
							target="_blank"
							class="text-sm text-[--color-link] hover:underline"
						>
							{{ formatMessage(messages.viewOnCurseForge) }}
						</a>
					</div>
				</template>
				<ProjectSidebarCreators
					:organization="null"
					:members="members"
					:org-link="isCurseForge ? () => '#' : (slug) => `https://modrinth.com/organization/${slug}`"
					:user-link="isCurseForge ? () => '#' : (username) => `https://modrinth.com/user/${username}`"
					link-target="_blank"
					class="project-sidebar-section"
				/>
				<ProjectSidebarDetails
					v-if="!isCurseForge"
					:project="data"
					:has-versions="versions.length > 0"
					:link-target="`_blank`"
					class="project-sidebar-section"
				/>
			</div>
		</Teleport>
		<div class="flex flex-col gap-4 p-6">
			<InstanceIndicator v-if="instance" :instance="instance" />
			<template v-if="data">
				<Teleport
					v-if="themeStore.featureFlags.project_background"
					to="#background-teleport-target"
				>
					<ProjectBackgroundGradient :project="data" />
				</Teleport>
				<ProjectHeader :project="data" @contextmenu.prevent.stop="handleRightClick">
					<template #actions>
						<ButtonStyled size="large" color="brand">
							<button
								v-tooltip="installed ? formatMessage(messages.alreadyInstalled) : null"
								:disabled="installed || installing"
								@click="install(null)"
							>
								<DownloadIcon v-if="!installed && !installing" />
								<CheckIcon v-else-if="installed" />
								{{
									installing
										? formatMessage(messages.installing)
										: installed
											? formatMessage(messages.installed)
											: formatMessage(messages.install)
								}}
							</button>
						</ButtonStyled>
						<ButtonStyled size="large" circular type="transparent">
							<OverflowMenu
								:tooltip="formatMessage(messages.moreOptions)"
								:options="[
									{
										id: 'follow',
										disabled: true,
										tooltip: formatMessage(messages.comingSoon),
										action: () => {},
									},
									{
										id: 'save',
										disabled: true,
										tooltip: formatMessage(messages.comingSoon),
										action: () => {},
									},
									{
										id: 'open-in-browser',
										link: openInBrowserLink,
										external: true,
									},
									{
										divider: true,
									},
									{
										id: 'report',
										color: 'red',
										hoverFilled: true,
										link: `https://modrinth.com/report?item=project&itemID=${data.id}`,
									},
								]"
								:aria-label="formatMessage(messages.moreOptions)"
							>
								<MoreVerticalIcon aria-hidden="true" />
								<template #open-in-browser>
									<ExternalIcon /> {{ formatMessage(messages.openInBrowser) }}
								</template>
								<template #follow> <HeartIcon /> {{ formatMessage(messages.follow) }} </template>
								<template #save> <BookmarkIcon /> {{ formatMessage(messages.save) }} </template>
								<template #report> <ReportIcon /> {{ formatMessage(messages.report) }} </template>
							</OverflowMenu>
						</ButtonStyled>
					</template>
				</ProjectHeader>
				<template v-if="isCurseForge">
					<NavTabs
						:links="[
							{
								label: formatMessage(messages.description),
								href: {
									path: `/project/${$route.params.id}`,
									query: curseForgeTabQuery,
								},
							},
							{
								label: `${formatMessage(messages.files)} (${cfFiles.length})`,
								href: {
									path: `/project/${$route.params.id}/versions`,
									query: curseForgeTabQuery,
								},
							},
						]"
					/>
					<div
						v-if="!$route.path.includes('/versions')"
						class="markdown-body"
						v-html="cfDescription || `<p class='text-secondary'>${formatMessage(messages.noDescription)}</p>`"
					/>
					<div v-else>
						<div v-if="cfFiles.length === 0" class="py-6 text-center text-secondary">
							{{ formatMessage(messages.noFilesFound) }}
						</div>
						<div v-else class="flex flex-col gap-2">
							<div
								v-for="f in cfFiles"
								:key="f.id"
								class="flex items-center gap-3 rounded-xl bg-[--color-raised-bg] p-3"
							>
								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-2">
										<span class="truncate font-semibold text-contrast">
											{{ f.displayName || f.fileName }}
										</span>
										<span
											class="shrink-0 rounded-full px-2 py-0.5 text-xs font-bold"
											:class="{
												'bg-green-500/15 text-green-500': (f.releaseType ?? 1) === 1,
												'bg-orange-500/15 text-orange-500': f.releaseType === 2,
												'bg-red-500/15 text-red-500': f.releaseType === 3,
											}"
										>
											{{ getCurseForgeReleaseType(f.releaseType) }}
										</span>
									</div>
									<div class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-secondary">
										<span v-if="(f.gameVersions ?? []).length">
											{{ (f.gameVersions ?? []).slice(0, 5).join(', ') }}
											<template v-if="(f.gameVersions ?? []).length > 5">
												+{{ (f.gameVersions?.length ?? 0) - 5 }} {{ formatMessage(messages.more) }}
											</template>
										</span>
										<span>{{ formatCurseForgeFileSize(f.fileLength) }}</span>
										<span v-if="f.fileDate">
											{{ dayjs(f.fileDate).fromNow() }}
										</span>
									</div>
								</div>
								<ButtonStyled color="brand" type="outlined">
									<button :disabled="installing" @click="installCfFile(f)">
										<DownloadIcon />
										{{ formatMessage(messages.install) }}
									</button>
								</ButtonStyled>
							</div>
						</div>
					</div>
				</template>
				<template v-else>
					<NavTabs
						:links="[
							{
								label: formatMessage(messages.description),
								href: `/project/${$route.params.id}`,
							},
							{
								label: formatMessage(messages.versions),
								href: {
									path: `/project/${$route.params.id}/versions`,
									query: instanceFilters,
								},
								subpages: ['version'],
							},
							{
								label: formatMessage(messages.gallery),
								href: `/project/${$route.params.id}/gallery`,
								shown: data.gallery.length > 0,
							},
						]"
					/>
					<RouterView
						:project="data"
						:versions="versions"
						:members="members"
						:instances="[]"
						:instance="instance"
						:install="install"
						:installed="installed"
						:installing="installing"
						:installed-version="installedVersion"
					/>
				</template>
			</template>
			<template v-else> {{ formatMessage(messages.projectLoadError) }} </template>
		</div>
		<ContextMenu ref="options" @option-clicked="handleOptionsClick">
			<template #install> <DownloadIcon /> {{ formatMessage(messages.install) }} </template>
			<template #open_link>
				<GlobeIcon /> {{ formatMessage(messages.openInModrinth) }} <ExternalIcon />
			</template>
			<template #copy_link> <ClipboardCopyIcon /> {{ formatMessage(messages.copyLink) }} </template>
		</ContextMenu>
	</div>
</template>

<script setup lang="ts">
import {
    BookmarkIcon,
    CheckIcon,
    ClipboardCopyIcon,
    DownloadIcon,
    ExternalIcon,
    GlobeIcon,
    HeartIcon,
    MoreVerticalIcon,
    ReportIcon,
} from '@modrinth/assets'
import {
    ButtonStyled,
    injectNotificationManager,
    OverflowMenu,
    ProjectBackgroundGradient,
    ProjectHeader,
    ProjectSidebarCompatibility,
    ProjectSidebarCreators,
    ProjectSidebarDetails,
    ProjectSidebarLinks,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onMounted, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import { get_project, get_team, get_version_many } from '@/helpers/cache.js'
import {
    constructCurseForgeCdnUrl,
    formatCurseForgeFileSize,
    getCurseForgeFileDownloadUrl,
    getCurseForgeMod,
    getCurseForgeModDescription,
    getCurseForgeModFiles,
    getCurseForgeProjectUrl,
    getCurseForgeReleaseType,
    type CurseForgeModFile,
} from '@/helpers/curseforge'
import {
    add_project_from_path,
    get as getInstance,
    get_projects as getInstanceProjects,
} from '@/helpers/profile'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { downloadUrlToTemp } from '@/helpers/utils'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { install as installVersion } from '@/store/install.js'
import { useTheming } from '@/store/state.js'

dayjs.extend(relativeTime)

const { handleError, addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()
const breadcrumbs = useBreadcrumbs()
const themeStore = useTheming()

const messages = defineMessages({
	curseForgeProject: { id: 'project.curseforge.heading', defaultMessage: 'CurseForge Project' },
	more: { id: 'project.more', defaultMessage: 'more' },
	viewOnCurseForge: { id: 'project.view-on-curseforge', defaultMessage: 'View on CurseForge' },
	alreadyInstalled: {
		id: 'project.install.already-installed',
		defaultMessage: 'This project is already installed',
	},
	installing: { id: 'project.install.installing', defaultMessage: 'Installing...' },
	installed: { id: 'project.install.installed', defaultMessage: 'Installed' },
	install: { id: 'project.install.action', defaultMessage: 'Install' },
	moreOptions: { id: 'project.more-options', defaultMessage: 'More options' },
	comingSoon: { id: 'project.coming-soon', defaultMessage: 'Coming soon' },
	openInBrowser: { id: 'project.action.open-in-browser', defaultMessage: 'Open in browser' },
	follow: { id: 'project.action.follow', defaultMessage: 'Follow' },
	save: { id: 'project.action.save', defaultMessage: 'Save' },
	report: { id: 'project.action.report', defaultMessage: 'Report' },
	description: { id: 'project.tab.description', defaultMessage: 'Description' },
	files: { id: 'project.tab.files', defaultMessage: 'Files' },
	noDescription: { id: 'project.no-description', defaultMessage: 'No description available.' },
	noFilesFound: { id: 'project.no-files-found', defaultMessage: 'No files found.' },
	versions: { id: 'project.tab.versions', defaultMessage: 'Versions' },
	gallery: { id: 'project.tab.gallery', defaultMessage: 'Gallery' },
	projectLoadError: {
		id: 'project.load-error',
		defaultMessage: "Project data couldn't not be loaded.",
	},
	openInModrinth: { id: 'project.action.open-in-modrinth', defaultMessage: 'Open in Modrinth' },
	copyLink: { id: 'project.action.copy-link', defaultMessage: 'Copy link' },
	manualDownloadRequiredTitle: {
		id: 'project.notification.manual-download-required.title',
		defaultMessage: 'Manual download required',
	},
	errorLoadingProject: { id: 'project.error.loading', defaultMessage: 'Error loading project' },
	breadcrumbProject: { id: 'project.breadcrumb', defaultMessage: 'Project' },
	selectInstanceFirst: {
		id: 'project.error.select-instance-first',
		defaultMessage: 'Select an instance first.',
	},
	curseForgeManualDownloadFallback: {
		id: 'project.notification.curseforge-manual-download-fallback',
		defaultMessage:
			'CurseForge could not provide a direct download. Opened the project page in your browser.',
	},
	curseForgeBlockedAutomatedDownload: {
		id: 'project.notification.curseforge-blocked-automated-download',
		defaultMessage:
			'CurseForge blocked automated download for this file (403). Opened the project page in your browser.',
	},
	noFilesAvailableToInstall: {
		id: 'project.error.no-files-available-to-install',
		defaultMessage: 'No files available to install.',
	},
})

const isCurseForge = computed(() => String(route.query.source ?? '') === 'curseforge')

const curseForgeTabQuery = computed(() => {
	const nextQuery: Record<string, any> = { ...route.query }
	nextQuery.source = 'curseforge'
	return nextQuery
})

const openInBrowserLink = computed(() => {
	if (!data.value) return ''
	if (isCurseForge.value) {
		return getCurseForgeProjectUrl(data.value.project_type, data.value.slug)
	}
	return `https://modrinth.com/${data.value.project_type}/${data.value.slug}`
})

function notifyCurseForgeManualDownload(project: any, message: string) {
	addNotification({
		title: formatMessage(messages.manualDownloadRequiredTitle),
		text: message,
		type: 'warning',
	})
	openUrl(getCurseForgeProjectUrl(project?.project_type, project?.slug))
}

const installing = ref(false)
const data = shallowRef<any>(null)
const versions = shallowRef<any[]>([])
const members = shallowRef<any[]>([])
const categories = shallowRef<any[]>([])
const cfFiles = ref<CurseForgeModFile[]>([])
const cfDescription = ref<string>('')
const instance = ref<any | null>(null)
const instanceProjects = ref<Record<string, any> | null>(null)

const installed = ref(false)
const installedVersion = ref<string | null>(null)

const instanceFilters = computed(() => {
	if (!instance.value || !data.value) {
		return {}
	}

	const loaders = []
	if (data.value?.project_type === 'mod') {
		if (instance.value.loader !== 'vanilla') {
			loaders.push(instance.value.loader)
		}
		if (instance.value.loader === 'vanilla' || data.value?.loaders?.includes?.('datapack')) {
			loaders.push('datapack')
		}
	}

	return { l: loaders, g: instance.value.game_version }
})

const allLoaders = ref<any[]>([])
const allGameVersions = ref<any[]>([])
const [loaderList, gameVersionList] = await Promise.all([
	get_loaders().catch(handleError),
	get_game_versions().catch(handleError),
])
allLoaders.value = (loaderList ?? []) as any[]
allGameVersions.value = (gameVersionList ?? []) as any[]

async function fetchProjectData() {
	installed.value = false
	installedVersion.value = null
	cfFiles.value = []
	cfDescription.value = ''

	const [fetchedInstance, fetchedInstanceProjects] = await Promise.all([
		route.query.i ? getInstance(route.query.i).catch(handleError) : Promise.resolve(null),
		route.query.i ? getInstanceProjects(route.query.i).catch(handleError) : Promise.resolve(null),
	])
	instance.value = fetchedInstance ?? null
	instanceProjects.value = fetchedInstanceProjects ?? null

	if (isCurseForge.value) {
		const modId = Number(route.params.id)
		const [mod, files, fetchedCategories, description] = await Promise.all([
			getCurseForgeMod(modId).catch(handleError),
			getCurseForgeModFiles({
				modId,
				index: 0,
				pageSize: 50,
				gameVersion: instance.value?.game_version,
			}).catch(handleError),
			get_categories().catch(handleError),
			getCurseForgeModDescription(modId).catch(() => ''),
		])

		categories.value = fetchedCategories ?? []
		cfFiles.value = files?.data ?? []
		cfDescription.value = description ?? ''

		if (!mod) {
			handleError(new Error(formatMessage(messages.errorLoadingProject)))
			data.value = null
			return
		}

		const cfCategories = (mod.categories ?? []).map((c) => c.name)
		const gameVersionsFromFiles = [
			...new Set(
				(files?.data ?? []).flatMap((f: CurseForgeModFile) => f.gameVersions ?? []),
			),
		].filter((v) => /^\d+\.\d+/.test(v))

		members.value = (mod.authors ?? []).map((a) => ({
			user: {
				username: a.name,
				avatar_url: null,
			},
			role: 'Author',
		}))
		versions.value = []

		data.value = {
			id: String(mod.id),
			slug: mod.slug ?? String(mod.id),
			title: mod.name,
			description: mod.summary,
			body: cfDescription.value,
			icon_url: mod.logo?.url ?? null,
			downloads: mod.downloadCount ?? 0,
			followers: mod.likeCount ?? 0,
			categories: cfCategories,
			status: 'approved',
			project_type: String(route.query.t ?? 'mod'),
			gallery: [],
			loaders: [],
			versions: [],
			client_side: 'unknown',
			server_side: 'unknown',
			game_versions: gameVersionsFromFiles.sort().reverse(),
			published: mod.dateCreated ?? null,
			updated: mod.dateModified ?? null,
			license: null,
			source_url: mod.links?.sourceUrl ?? null,
			issues_url: mod.links?.issuesUrl ?? null,
			wiki_url: mod.links?.wikiUrl ?? null,
			link: mod.links?.websiteUrl ?? getCurseForgeProjectUrl(String(route.query.t ?? 'mod'), mod.slug),
		}
		breadcrumbs.setName(formatMessage(messages.breadcrumbProject), data.value.title)
		return
	}

	const project = await get_project(route.params.id, 'must_revalidate').catch(handleError)

	if (!project) {
		handleError(new Error(formatMessage(messages.errorLoadingProject)))
		return
	}

	data.value = project
	const [fetchedVersions, fetchedMembers, fetchedCategories] = await Promise.all([
		get_version_many(project.versions, 'must_revalidate').catch(handleError),
		get_team(project.team).catch(handleError),
		get_categories().catch(handleError),
	])

	versions.value = fetchedVersions ?? []
	members.value = fetchedMembers ?? []
	categories.value = fetchedCategories ?? []
	versions.value = versions.value
		.slice()
		.sort((a, b) => dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf())

	if (instanceProjects.value && data.value) {
		const installedFile = Object.values(instanceProjects.value).find(
			(x) => x.metadata && x.metadata.project_id === data.value.id,
		)
		if (installedFile) {
			installed.value = true
			installedVersion.value = installedFile.metadata.version_id
		}
	}
	breadcrumbs.setName(formatMessage(messages.breadcrumbProject), data.value.title)
}

onMounted(async () => {
	await fetchProjectData()
})

watch(
	() => route.params.id,
	async () => {
		if (route.params.id && route.path.startsWith('/project')) {
			await fetchProjectData()
		}
	},
)

async function installCfFile(file: CurseForgeModFile) {
	if (!instance.value?.path) {
		handleError(new Error(formatMessage(messages.selectInstanceFirst)))
		return
	}
	try {
		const modId = Number(route.params.id)
		let downloadUrl = file.downloadUrl || null
		if (!downloadUrl) {
			downloadUrl = await getCurseForgeFileDownloadUrl(modId, file.id).catch(() => null)
		}
		if (!downloadUrl && file.id && file.fileName) {
			downloadUrl = constructCurseForgeCdnUrl(file.id, file.fileName)
		}
		if (!downloadUrl) {
			notifyCurseForgeManualDownload(
				data.value,
				formatMessage(messages.curseForgeManualDownloadFallback),
			)
			return
		}

		const suggestedName = (file.fileName || `curseforge-${modId}-${file.id}.jar`).replace(
			/[^a-zA-Z0-9._-]/g,
			'_',
		)
		const downloadedPath = await downloadUrlToTemp(downloadUrl, suggestedName, instance.value.path)
		const rawType = String(route.query.t ?? 'mod')
		const backendType = rawType === 'shader' ? 'shaderpack' : rawType
		await add_project_from_path(instance.value.path, downloadedPath, backendType)
		installed.value = true
	} catch (e) {
		const err = e instanceof Error ? e : new Error(String(e))
		if (String(err.message ?? '').includes('(403)') || String(err.message ?? '').includes('403')) {
			notifyCurseForgeManualDownload(
				data.value,
				formatMessage(messages.curseForgeBlockedAutomatedDownload),
			)
			return
		}
		handleError(err)
	}
}

async function install(version?: string | null) {
	installing.value = true
	if (isCurseForge.value) {
		try {
			if (cfFiles.value.length === 0) {
				handleError(new Error(formatMessage(messages.noFilesAvailableToInstall)))
				return
			}
			const first = cfFiles.value[0]
			await installCfFile(first)
		} finally {
			installing.value = false
		}
		return
	}
	await (installVersion as any)(
		data.value.id,
		version,
		instance.value ? instance.value.path : null,
		'ProjectPage',
		() => {
			installing.value = false

			if (instance.value && version) {
				installed.value = true
				installedVersion.value = version
			}
		},
		() => {},
	).catch(handleError)
}

const options = ref<any | null>(null)
const handleRightClick = (event: MouseEvent) => {
	options.value?.showMenu(event, data.value, [
		{
			name: 'install',
		},
		{
			type: 'divider',
		},
		{
			name: 'open_link',
		},
		{
			name: 'copy_link',
		},
	])
}
const handleOptionsClick = (args: any) => {
	switch (args.option) {
		case 'install':
			install(null)
			break
		case 'open_link':
			if (isCurseForge.value) {
				openUrl(getCurseForgeProjectUrl(args.item.project_type, args.item.slug))
			} else {
				openUrl(`https://modrinth.com/${args.item.project_type}/${args.item.slug}`)
			}
			break
		case 'copy_link':
			if (isCurseForge.value) {
				navigator.clipboard.writeText(
					getCurseForgeProjectUrl(args.item.project_type, args.item.slug),
				)
			} else {
				navigator.clipboard.writeText(
					`https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
				)
			}
			break
	}
}
</script>

<style scoped lang="scss">
.root-container {
	display: flex;
	flex-direction: row;
	min-height: 100%;
}

.project-sidebar {
	position: fixed;
	width: calc(300px + 1.5rem);
	min-height: calc(100vh - 3.25rem);
	height: fit-content;
	max-height: calc(100vh - 3.25rem);
	padding: 1rem 0.5rem 1rem 1rem;
	overflow-y: auto;
	-ms-overflow-style: none;
	scrollbar-width: none;

	&::-webkit-scrollbar {
		width: 0;
		background: transparent;
	}
}

.sidebar-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.content-container {
	display: flex;
	flex-direction: column;
	width: 100%;
	padding: 1rem;
	margin-left: calc(300px + 1rem);
}

.button-group {
	display: flex;
	flex-wrap: wrap;
	flex-direction: row;
	gap: 0.5rem;
}

.stats {
	display: flex;
	flex-direction: column;
	flex-wrap: wrap;
	gap: var(--gap-md);

	.stat {
		display: flex;
		flex-direction: row;
		align-items: center;
		width: fit-content;
		gap: var(--gap-xs);
		--stat-strong-size: 1.25rem;

		strong {
			font-size: var(--stat-strong-size);
		}

		p {
			margin: 0;
		}

		svg {
			min-height: var(--stat-strong-size);
			min-width: var(--stat-strong-size);
		}
	}

	.date {
		margin-top: auto;
	}
}

.tabs {
	display: flex;
	flex-direction: row;
	gap: 1rem;
	margin-bottom: var(--gap-md);
	justify-content: space-between;

	.tab {
		display: flex;
		flex-direction: row;
		align-items: center;
		border-radius: var(--border-radius);
		cursor: pointer;
		transition: background-color 0.2s ease-in-out;

		&:hover {
			background-color: var(--color-raised-bg);
		}

		&.router-view-active {
			background-color: var(--color-raised-bg);
		}
	}
}

.links {
	a {
		display: inline-flex;
		align-items: center;
		border-radius: 1rem;
		color: var(--color-text);

		svg,
		img {
			height: 1rem;
			width: 1rem;
		}

		span {
			margin-left: 0.25rem;
			text-decoration: underline;
			line-height: 2rem;
		}

		&:focus-visible,
		&:hover {
			svg,
			img,
			span {
				color: var(--color-heading);
			}
		}

		&:active {
			svg,
			img,
			span {
				color: var(--color-text-dark);
			}
		}

		&:not(:last-child)::after {
			content: '•';
			margin: 0 0.25rem;
		}
	}
}

.install-loading {
	scale: 0.2;
	height: 1rem;
	width: 1rem;
	margin-right: -1rem;

	:deep(svg) {
		color: var(--color-contrast);
	}
}

.project-sidebar-section {
	@apply p-4 flex flex-col gap-2 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid;
}

.markdown-body {
	:deep(img) {
		max-width: 100%;
		height: auto;
		border-radius: var(--radius-md);
	}

	:deep(ul),
	:deep(ol) {
		margin-left: 2rem;
	}

	:deep(a) {
		color: var(--color-link);

		&:hover {
			text-decoration: underline;
		}
	}

	:deep(h1),
	:deep(h2),
	:deep(h3),
	:deep(h4) {
		color: var(--color-contrast);
	}

	:deep(hr) {
		border-color: var(--color-button-bg);
	}

	:deep(p) {
		line-height: 1.6;
	}
}
</style>
