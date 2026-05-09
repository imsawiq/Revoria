import { spawn } from 'node:child_process'

const localBypass = ['localhost', '127.0.0.1', '::1']
const mergeBypass = (value) => {
	const values = new Set(
		String(value || '')
			.split(',')
			.map((entry) => entry.trim())
			.filter(Boolean),
	)
	for (const entry of localBypass) {
		values.add(entry)
	}
	return [...values].join(',')
}

const env = {
	...process.env,
	NO_PROXY: mergeBypass(process.env.NO_PROXY),
}

if (process.platform !== 'win32') {
	env.no_proxy = mergeBypass(process.env.no_proxy)
} else {
	delete env.no_proxy
}

const child = spawn('pnpm exec tauri dev', {
	env,
	stdio: 'inherit',
	shell: true,
})

child.on('exit', (code, signal) => {
	if (signal) {
		process.kill(process.pid, signal)
		return
	}
	process.exit(code ?? 0)
})
