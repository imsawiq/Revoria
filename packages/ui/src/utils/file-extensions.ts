export const FILE_CODE_EXTENSIONS = [
	'json',
	'json5',
	'jsonc',
	'java',
	'kt',
	'kts',
	'sh',
	'bat',
	'ps1',
	'yml',
	'yaml',
	'toml',
	'js',
	'ts',
	'py',
	'rb',
	'php',
	'html',
	'css',
	'cpp',
	'c',
	'h',
	'rs',
	'go',
] as const

export const FILE_TEXT_EXTENSIONS = [
	'txt',
	'md',
	'log',
	'cfg',
	'conf',
	'properties',
	'ini',
	'sk',
] as const

export const FILE_IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'] as const

export const FILE_ARCHIVE_EXTENSIONS = ['zip', 'jar', 'tar', 'gz', 'rar', '7z'] as const

export type CodeExtension = (typeof FILE_CODE_EXTENSIONS)[number]
export type TextExtension = (typeof FILE_TEXT_EXTENSIONS)[number]
export type ImageExtension = (typeof FILE_IMAGE_EXTENSIONS)[number]
export type ArchiveExtension = (typeof FILE_ARCHIVE_EXTENSIONS)[number]

export function getFileExtension(filename: string): string {
	return filename.split('.').pop()?.toLowerCase() ?? ''
}

export function isCodeFile(ext: string): boolean {
	return (FILE_CODE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

export function isTextFile(ext: string): boolean {
	return (FILE_TEXT_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

export function isImageFile(ext: string): boolean {
	return (FILE_IMAGE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

export function isArchiveFile(ext: string): boolean {
	return (FILE_ARCHIVE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

export function isEditableFile(ext: string): boolean {
	return isCodeFile(ext) || isTextFile(ext)
}

export function getEditorLanguage(ext: string): string {
	const lowered = ext.toLowerCase()
	switch (lowered) {
		case 'json':
		case 'json5':
		case 'jsonc':
			return 'json'
		case 'toml':
			return 'toml'
		case 'sh':
			return 'sh'
		case 'bat':
			return 'batchfile'
		case 'ps1':
			return 'powershell'
		case 'yml':
		case 'yaml':
			return 'yaml'
		case 'js':
			return 'javascript'
		case 'ts':
			return 'typescript'
		case 'py':
			return 'python'
		case 'rb':
			return 'ruby'
		case 'php':
			return 'php'
		case 'html':
			return 'html'
		case 'css':
			return 'css'
		case 'java':
		case 'kt':
		case 'kts':
			return 'java'
		case 'cpp':
		case 'c':
		case 'h':
			return 'c_cpp'
		case 'rs':
			return 'rust'
		case 'go':
			return 'golang'
		case 'md':
			return 'markdown'
		case 'properties':
			return 'properties'
		case 'ini':
		case 'cfg':
		case 'conf':
			return 'ini'
		case 'log':
			return 'mclog'
		default:
			return 'text'
	}
}
