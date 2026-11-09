#!/usr/bin/env bash
# Fails if a member Cargo.toml declares a dependency version instead of inheriting it from the root [workspace.dependencies].

cd "$(git rev-parse --show-toplevel)" || exit 1

FOUND_ERROR=0
for file in $(git ls-files '*Cargo.toml' ':!Cargo.toml' ':!**/vendored/**'); do
	RESULT=$(awk '
		/^\[/ { in_deps = ($0 ~ /^\[(.*\.)?(dev-|build-)?dependencies(\.|\])/) }
		in_deps && !/^[[:space:]]*#/ && !/workspace[[:space:]]*=[[:space:]]*true/ && /=/ { print }
	' "$file")
	if [ -n "$RESULT" ]; then
		echo "Dependencies not using the workspace version in $file:"
		echo "$RESULT"
		FOUND_ERROR=1
	fi
done

[ $FOUND_ERROR -eq 0 ] && echo "All member Cargo.toml dependencies inherit from the workspace"
exit $FOUND_ERROR
