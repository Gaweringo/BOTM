set shell := ["nu", "-c"]

release-dry LEVEL:
    cargo release version {{LEVEL}}
    cargo release tag
    # standard-version --skip.tag --dry-run
    git-cliff

release LEVEL:
    cargo release version {{LEVEL}} -x
    cargo release hook -x --no-confirm
    git add .
    cargo release commit -x --no-confirm
    cargo release tag -x --no-confirm
    # standard-version --skip.tag --dry-run
    # git-cliff -o CHANGELOG.md