#!/usr/bin/env bash
# Note di release per GitHub: i percorsi relativi di docs/RELEASE-<versione>.md (logo, CHANGELOG,
# VERIFICA) non funzionano nella pagina della release, quindi diventano URL assoluti del repository.
set -euo pipefail
version="$1"
repo="https://github.com/${GITHUB_REPOSITORY:-Kyo89-sudo/AmbraMusic}/blob/main"
sed -e "s#src=\"\.\./\([^\"]*\.png\)\"#src=\"$repo/\1?raw=true\"#g" \
    -e "s#](\.\./\([^)]*\))#]($repo/\1)#g" \
    -e "s#](\([A-Za-z][^):]*\.\(md\|png\)\))#]($repo/docs/\1)#g" \
    -e "s#src=\"\(images/[^\"]*\)\"#src=\"$repo/docs/\1?raw=true\"#g" \
    "docs/RELEASE-$version.md"
