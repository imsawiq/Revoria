<template>
	<div>
		<Card>
			<Breadcrumbs
				:current-title="version.name"
				:link-stack="[
					{
						href: `/project/${route.params.id}/versions`,
						label: formatMessage(messages.versions),
					},
				]"
			/>
			<div class="version-title">
				<h2>{{ version.name }}</h2>
			</div>
			<div class="button-group">
				<Button
					color="primary"
					:action="() => install(version.id)"
					:disabled="installing || (installed && installedVersion === version.id)"
				>
					<DownloadIcon v-if="!installed" />
					<SwapIcon v-else-if="installedVersion !== version.id" />
					<CheckIcon v-else />
					{{
						installing
							? formatMessage(messages.installing)
							: installed && installedVersion === version.id
								? formatMessage(messages.installed)
								: formatMessage(messages.install)
					}}
				</Button>
				<Button>
					<ReportIcon />
					{{ formatMessage(messages.report) }}
				</Button>
				<a
					:href="`https://modrinth.com/mod/${route.params.id}/version/${route.params.version}`"
					rel="external"
					class="btn"
				>
					<ExternalIcon />
					{{ formatMessage(messages.modrinthWebsite) }}
				</a>
			</div>
		</Card>
		<div class="version-container">
			<div class="description-cards">
				<Card>
					<h3 class="card-title">{{ formatMessage(messages.changelog) }}</h3>
					<div class="markdown-body" v-html="renderString(version.changelog ?? '')" />
				</Card>
				<Card>
					<h3 class="card-title">{{ formatMessage(messages.files) }}</h3>
					<Card
						v-for="file in version.files"
						:key="file.id"
						:class="{ primary: file.primary }"
						class="file"
					>
						<span class="label">
							<FileIcon />
							<span>
								<span class="title">
									{{ file.filename }}
								</span>
								({{ formatBytes(file.size) }})
								<span v-if="file.primary" class="primary-label"> {{ formatMessage(messages.primary) }} </span>
							</span>
						</span>
						<Button
							v-if="project.project_type !== 'modpack' || file.primary"
							class="download"
							:action="() => install(version.id)"
							:disabled="installed"
						>
							<DownloadIcon v-if="!installed" />
							<CheckIcon v-else />
							{{ installed ? formatMessage(messages.installed) : formatMessage(messages.install) }}
						</Button>
					</Card>
				</Card>
				<Card v-if="displayDependencies.length > 0">
					<h2>{{ formatMessage(messages.dependencies) }}</h2>
					<div v-for="dependency in displayDependencies" :key="dependency.title">
						<router-link v-if="dependency.link" class="btn dependency" :to="dependency.link">
							<Avatar size="sm" :src="dependency.icon" />
							<div>
								<span class="title"> {{ dependency.title }} </span> <br />
								<span> {{ dependency.subtitle }} </span>
							</div>
						</router-link>
						<div v-else class="dependency disabled" disabled="">
							<Avatar size="sm" :src="dependency.icon" />
							<div class="text">
								<div class="title">{{ dependency.title }}</div>
								<div>{{ dependency.subtitle }}</div>
							</div>
						</div>
					</div>
				</Card>
			</div>
			<Card class="metadata-card">
				<h3 class="card-title">{{ formatMessage(messages.metadata) }}</h3>
				<div class="metadata">
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.releaseChannel) }}</span>
						<span class="metadata-value"
							><Badge
								:color="releaseColor(version.version_type)"
								:type="
									version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)
								"
						/></span>
					</div>
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.versionNumber) }}</span>
						<span class="metadata-value">{{ version.version_number }}</span>
					</div>
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.loaders) }}</span>
						<span class="metadata-value">{{
							version.loaders
								.map((loader) => loader.charAt(0).toUpperCase() + loader.slice(1))
								.join(', ')
						}}</span>
					</div>
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.gameVersions) }}</span>
						<span class="metadata-value"> {{ version.game_versions.join(', ') }} </span>
					</div>
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.downloads) }}</span>
						<span class="metadata-value">{{ version.downloads }}</span>
					</div>
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.publicationDate) }}</span>
						<span class="metadata-value">
							{{
								new Date(version.date_published).toLocaleString('en-US', {
									month: 'long',
									day: 'numeric',
									year: 'numeric',
								})
							}}
							{{ ' ' }}
							{{ formatMessage(messages.at) }}
							{{ ' ' }}
							{{
								new Date(version.date_published).toLocaleString('en-US', {
									hour: 'numeric',
									minute: 'numeric',
									second: 'numeric',
									hour12: true,
								})
							}}
						</span>
					</div>
					<div v-if="author" class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.author) }}</span>
						<a
							:href="`https://modrinth.com/user/${author.user.username}`"
							rel="external"
							class="metadata-value btn author"
						>
							<Avatar size="sm" :src="author.user.avatar_url" circle />
							<span>
								<strong>
									{{ author.user.username }}
								</strong>
								<br />
								{{ author.role }}
							</span>
						</a>
					</div>
					<div class="metadata-item">
						<span class="metadata-label">{{ formatMessage(messages.versionId) }}</span>
						<span class="metadata-value"><CopyCode class="copycode" :text="version.id" /></span>
					</div>
				</div>
			</Card>
		</div>
	</div>
</template>

<script setup>
import { CheckIcon, DownloadIcon, ExternalIcon, FileIcon, ReportIcon } from '@modrinth/assets'
import { Avatar, Badge, Breadcrumbs, Button, Card, CopyCode } from '@modrinth/ui'
import { formatBytes, renderString } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { SwapIcon } from '@/assets/icons'
import { get_project_many, get_version_many } from '@/helpers/cache.js'
import { releaseColor } from '@/helpers/utils'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const breadcrumbs = useBreadcrumbs()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	versions: { id: 'project.tab.versions', defaultMessage: 'Versions' },
	installing: { id: 'project.install.installing', defaultMessage: 'Installing...' },
	installed: { id: 'project.install.installed', defaultMessage: 'Installed' },
	install: { id: 'project.install.action', defaultMessage: 'Install' },
	report: { id: 'project.action.report', defaultMessage: 'Report' },
	modrinthWebsite: { id: 'project.modrinth-website', defaultMessage: 'Modrinth website' },
	changelog: { id: 'project.changelog', defaultMessage: 'Changelog' },
	files: { id: 'project.tab.files', defaultMessage: 'Files' },
	primary: { id: 'project.file.primary', defaultMessage: 'Primary' },
	dependencies: { id: 'project.dependencies', defaultMessage: 'Dependencies' },
	metadata: { id: 'project.metadata', defaultMessage: 'Metadata' },
	releaseChannel: { id: 'project.metadata.release-channel', defaultMessage: 'Release Channel' },
	versionNumber: { id: 'project.metadata.version-number', defaultMessage: 'Version Number' },
	loaders: { id: 'project.metadata.loaders', defaultMessage: 'Loaders' },
	gameVersions: { id: 'project.metadata.game-versions', defaultMessage: 'Game Versions' },
	downloads: { id: 'project.metadata.downloads', defaultMessage: 'Downloads' },
	publicationDate: { id: 'project.metadata.publication-date', defaultMessage: 'Publication Date' },
	at: { id: 'project.metadata.at', defaultMessage: 'at' },
	author: { id: 'project.metadata.author', defaultMessage: 'Author' },
	versionId: { id: 'project.metadata.version-id', defaultMessage: 'Version ID' },
	breadcrumbVersion: { id: 'project.breadcrumb.version', defaultMessage: 'Version' },
	dependencyVersionIsType: {
		id: 'project.dependency.version-is-type',
		defaultMessage: 'Version {version} is {type}',
	},
	addedViaOverrides: {
		id: 'project.dependency.added-via-overrides',
		defaultMessage: 'Added via overrides',
	},
})

const route = useRoute()

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	versions: {
		type: Array,
		required: true,
	},
	members: {
		type: Array,
		required: true,
	},
	install: {
		type: Function,
		required: true,
	},
	installed: {
		type: Boolean,
		required: true,
	},
	installing: {
		type: Boolean,
		required: true,
	},
	installedVersion: {
		type: String,
		required: true,
	},
})

const version = ref(props.versions.find((version) => version.id === route.params.version))
breadcrumbs.setName(formatMessage(messages.breadcrumbVersion), version.value.name)

watch(
	() => props.versions,
	async () => {
		if (route.params.version) {
			version.value = props.versions.find((version) => version.id === route.params.version)
			await refreshDisplayDependencies()
			breadcrumbs.setName(formatMessage(messages.breadcrumbVersion), version.value.name)
		}
	},
)

const author = computed(() =>
	props.members ? props.members.find((member) => member.user.id === version.value.author_id) : null,
)

const displayDependencies = ref({})

async function refreshDisplayDependencies() {
	const projectIds = new Set()
	const versionIds = new Set()
	if (version.value.dependencies) {
		for (const dependency of version.value.dependencies) {
			if (dependency.project_id) {
				projectIds.add(dependency.project_id)
			}
			if (dependency.version_id) {
				versionIds.add(dependency.version_id)
			}
		}
	}
	const [projectDeps, versionDeps] = await Promise.all([
		get_project_many([...projectIds]),
		get_version_many([...versionIds]),
	])

	const dependencies = {
		projects: projectDeps,
		versions: versionDeps,
	}

	displayDependencies.value = version.value.dependencies.map((dependency) => {
		const version = dependencies.versions.find((obj) => obj.id === dependency.version_id)
		if (version) {
			const project = dependencies.projects.find(
				(obj) => obj.id === version.project_id || obj.id === dependency.project_id,
			)
			return {
				icon: project?.icon_url,
				title: project?.title || project?.name,
				subtitle: formatMessage(messages.dependencyVersionIsType, {
					version: version.version_number,
					type: dependency.dependency_type,
				}),
				link: `/project/${project.slug}/version/${version.id}`,
			}
		} else {
			const project = dependencies.projects.find((obj) => obj.id === dependency.project_id)

			if (project) {
				return {
					icon: project?.icon_url,
					title: project?.title || project?.name,
					subtitle: `${dependency.dependency_type}`,
					link: `/project/${project.slug}`,
				}
			} else {
				return {
					icon: null,
					title: dependency.file_name,
					subtitle: formatMessage(messages.addedViaOverrides),
					link: null,
				}
			}
		}
	})
}
onMounted(async () => {
	await refreshDisplayDependencies()
})
</script>

<style scoped lang="scss">
.version-container {
	display: flex;
	flex-direction: row;
	gap: 1rem;
}

.version-title {
	margin-bottom: 1rem;
	h2 {
		font-size: var(--font-size-2xl);
		font-weight: 700;
		color: var(--color-contrast);
		margin: 0;
	}
}

.dependency {
	display: flex;
	padding: 0.5rem 1rem 0.5rem 0.5rem;
	gap: 0.5rem;
	background: var(--color-raised-bg);
	color: var(--color-base);
	width: 100%;

	.title {
		font-weight: bolder;
	}

	:deep(svg) {
		margin-right: 0 !important;
	}
}

.file {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
	background: var(--color-button-bg);
	color: var(--color-base);
	padding: 0.5rem 1rem;

	.download {
		margin-left: auto;
		background-color: var(--color-raised-bg);
	}

	.label {
		display: flex;
		margin: auto 0 auto;
		gap: 0.5rem;

		.title {
			font-weight: bolder;
			word-break: break-all;
		}

		svg {
			min-width: 1.1rem;
			min-height: 1.1rem;
			width: 1.1rem;
			height: 1.1rem;
			margin: auto 0;
		}

		.primary-label {
			font-style: italic;
		}
	}
}

.primary {
	background: var(--color-brand-highlight);
	color: var(--color-contrast);
}

.button-group {
	display: flex;
	flex-wrap: wrap;
	flex-direction: row;
	gap: 0.5rem;
}

.card-title {
	font-size: var(--font-size-lg);
	color: var(--color-contrast);
	margin: 0 0 0.5rem;
}

.description-cards {
	width: 100%;
}

.metadata-card {
	width: 20rem;
	height: min-content;
}

.metadata {
	display: flex;
	flex-direction: column;
	flex-wrap: wrap;
	gap: 1rem;

	.metadata-item {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		.metadata-label {
			font-weight: bold;
		}
	}
}

.author {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
	align-items: center;
	text-decoration: none;
	color: var(--color-base);
	background: var(--color-raised-bg);
	padding: 0.5rem;
	width: 100%;
	box-shadow: none;
}

.markdown-body {
	:deep(hr),
	:deep(h1),
	:deep(h2),
	img {
		max-width: max(60rem, 90%) !important;
	}

	:deep(ul),
	:deep(ol) {
		margin-left: 2rem;
	}
}

.copycode {
	border: 0;
	color: var(--color-contrast);
}

.disabled {
	display: flex;
	flex-direction: row;
	vertical-align: center;
	align-items: center;
	cursor: not-allowed;
	border-radius: var(--radius-lg);

	.text {
		opacity: 0.6;
	}
}
</style>
