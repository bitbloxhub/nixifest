#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
output_dir="$repo_root/modules/nixifest/generated"
first_minor=${KUBERNETES_FIRST_MINOR:-27}
jobs=${KUBERNETES_JOBS:-$(nproc)}
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT
staging_dir="$tmp_dir/generated"
mkdir -p "$staging_dir"
typegen=$(nix build "$repo_root#typegen" --no-link --print-out-paths)

declare -a releases=()
declare -a pids=()
latest_minor=

for minor in $(seq "$first_minor" 99); do
	if ! version=$(curl --fail --silent --show-error --retry 3 "https://dl.k8s.io/release/stable-1.${minor}.txt" 2>/dev/null); then
		if ((minor > first_minor)); then
			break
		fi
		printf 'no stable Kubernetes release found for 1.%s\n' "$minor" >&2
		exit 1
	fi
	releases+=("${minor}:${version}")
	latest_minor=$minor
done

format_file() {
	local file=$1
	printf 'formatting %s\n' "${file##*/}"
	nixfmt "$file"
	deadnix --edit "$file"
	statix fix "$file"
}

generate() {
	local minor=$1
	local version=$2
	local source_dir="$tmp_dir/$version"
	local output="$staging_dir/v1_${minor}.nix"
	if [[ "${KUBERNETES_FORCE:-0}" != 1 && -e "$output_dir/v1_${minor}.nix" ]] && [[ "$(head -n1 "$output_dir/v1_${minor}.nix")" == "# Kubernetes $version" ]]; then
		printf 'reusing %s\n' "$output_dir/v1_${minor}.nix"
		cp "$output_dir/v1_${minor}.nix" "$output"
		return
	fi

	mkdir -p "$source_dir"
	printf 'generating Kubernetes %s\n' "$version"
	curl --fail --silent --show-error --retry 3 "https://dl.k8s.io/release/$version/kubernetes-src.tar.gz" | tar --extract --gzip --file - --directory "$source_dir"
	raw_output="$output.raw"
	"$typegen/bin/nixifest-typegen" kubernetes --input "$source_dir/api" --output "$raw_output"
	{
		printf '# Kubernetes %s\n' "$version"
		cat "$raw_output"
	} >"$output"
	rm "$raw_output"
	format_file "$output"
}

for release in "${releases[@]}"; do
	IFS=: read -r minor version <<<"$release"
	generate "$minor" "$version" &
	pids+=("$!")
	if ((${#pids[@]} >= jobs)); then
		wait "${pids[0]}"
		pids=("${pids[@]:1}")
	fi
done
for pid in "${pids[@]}"; do
	wait "$pid"
done

{
	printf '%s\n' '{'
	for release in "${releases[@]}"; do
		IFS=: read -r minor _ <<<"$release"
		printf '  v1_%s = ./v1_%s.nix;\n' "$minor" "$minor"
	done
	printf '  latest = ./v1_%s.nix;\n' "$latest_minor"
	printf '%s\n' '}'
} >"$staging_dir/default.nix"

cp "$staging_dir"/*.nix "$output_dir"/
