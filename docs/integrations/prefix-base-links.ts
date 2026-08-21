type PrefixBaseLinksOptions = {
	base: string
}

type MarkdownNode = {
	type?: string
	url?: string
	children?: MarkdownNode[]
}

export function prefixBaseLinks({ base }: PrefixBaseLinksOptions) {
	const prefix = base.endsWith('/') ? base.slice(0, -1) : base

	return (tree: MarkdownNode) => {
		const visit = (node: MarkdownNode) => {
			if (node.type === 'link' && node.url?.startsWith('/') && !node.url.startsWith(`${prefix}/`)) {
				node.url = `${prefix}${node.url}`
			}

			for (const child of node.children ?? []) {
				visit(child)
			}
		}

		visit(tree)
	}
}
