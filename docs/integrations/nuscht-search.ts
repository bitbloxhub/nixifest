import type { AstroIntegration } from 'astro'
import { execFile } from 'node:child_process'
import { chmod, cp, mkdir, readFile, writeFile } from 'node:fs/promises'
import { promisify } from 'node:util'
import { fileURLToPath } from 'node:url'

const execFileAsync = promisify(execFile)

const themeCss = `
@import url('https://fonts.googleapis.com/css2?family=Fira+Code:wght@300..700&display=swap');

:root {
  /* Catppuccin Mocha: mauve primary, blue accent. */
  color-scheme: dark;
  --c-primary: #cba6f7; /* mauve */
  --c-primary-l: #f5c2e7; /* pink */
  --c-primary-d: #b4befe; /* lavender */
  --c-accent: #89b4fa; /* blue */
  --c-accent-l: #b4befe; /* lavender */
  --c-accent-d: #74c7ec; /* sapphire */
  --c-secondary: #1e1e2e; /* base */
  --c-secondary-l: #313244; /* surface0 */
  --c-secondary-ll: #45475a; /* surface1 */
  --c-secondary-d: #11111b; /* crust */
  --c-ternary: #cdd6f4; /* text */
  --c-ternary-l: #f5e0e6; /* rosewater */
  --c-ternary-ll: #bac2de; /* subtext1 */
  --c-ternary-d: #a6adc8; /* subtext0 */
  --c-gray: #6c7086; /* overlay0 */
  --c-good: #a6e3a1; /* green */
  --c-good-l: #94e2d5; /* teal */
  --c-good-d: #a6e3a1; /* green */
  --c-danger: #f38ba8; /* red */
  --c-danger-l: #eba0ac; /* maroon */
  --c-danger-d: #f38ba8; /* red */
  --f-family: 'Fira Code', monospace;
  --f-family-mono: 'Fira Code', monospace;
}

@media (prefers-color-scheme: light) {
  /* Catppuccin Latte: mauve primary, blue accent. */
  :root {
    color-scheme: light;
    --c-primary: #8839ef; /* mauve */
    --c-primary-l: #ea76cb; /* pink */
    --c-primary-d: #7287fd; /* lavender */
    --c-accent: #1e66f5; /* blue */
    --c-accent-l: #7287fd; /* lavender */
    --c-accent-d: #209fb5; /* sapphire */
    --c-secondary: #eff1f5; /* base */
    --c-secondary-l: #e6e9ef; /* mantle */
    --c-secondary-ll: #ccd0da; /* surface0 */
    --c-secondary-d: #dce0e8; /* crust */
    --c-ternary: #4c4f69; /* text */
    --c-ternary-l: #ffffff; /* white */
    --c-ternary-ll: #dce0e8; /* crust */
    --c-ternary-d: #5c5f77; /* subtext1 */
    --c-gray: #9ca0b0; /* overlay0 */
    --c-good: #40a02b; /* green */
    --c-good-l: #179299; /* teal */
    --c-good-d: #40a02b; /* green */
    --c-danger: #d20f39; /* red */
    --c-danger-l: #e64553; /* maroon */
    --c-danger-d: #d20f39; /* red */
  }
}

body.dark {
  color-scheme: dark;
}
`

export function nuschtSearch(): AstroIntegration {
	return {
		name: 'nixifest-nuscht-search',
		hooks: {
			'astro:build:done': async ({ dir, logger }) => {
				let source = process.env.NUSCHT_SEARCH_PATH

				if (!source) {
					logger.info('building Nixifest option search')
					const { stdout } = await execFileAsync('nix', [
						'build',
						'.#search',
						'--no-link',
						'--print-out-paths',
					])
					source = stdout.trim()
				}

				if (!source) {
					throw new Error('Nixifest option search build returned no path')
				}

				const destination = fileURLToPath(new URL('./options/search/', dir))
				await mkdir(destination, { recursive: true })
				await cp(source, destination, { recursive: true })
				await chmod(destination, 0o755, { recursive: true })
				await writeFile(`${destination}/nixifest-theme.css`, themeCss)
				const indexPath = `${destination}/index.html`
				await chmod(indexPath, 0o644)
				const index = await readFile(indexPath, 'utf8')
				await writeFile(
					indexPath,
					index.replace('</head>', '  <link rel="stylesheet" href="nixifest-theme.css"></head>'),
				)
				logger.info(`copied Nixifest option search to ${destination}`)
			},
		},
	}
}
