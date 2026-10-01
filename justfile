output-schemata-path := "schemata/generated"

generate-all: generate-schemata

generate-schemata:
    rm -rf {{ output-schemata-path }}
    mkdir -p {{ output-schemata-path }}
    cargo run --package openfx-datagen --bin cli -- gen-schemata --output-schemata "{{ output-schemata-path }}"

detect-stale-generated-contents: generate-all
    #!/usr/bin/env sh
    if ! git diff --quiet; then
        echo "Stale generated contents detected."
        echo "Please run \`just generate-all\` and commit the changes."
        echo "stale files:"
        git --no-pager diff --name-only
        exit 1
    fi
