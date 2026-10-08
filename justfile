output-schemata-path := "schemata/generated"

generate-all: generate-schemata generate-reference-bindings

generate-schemata:
    rm -rf {{ output-schemata-path }}
    mkdir -p {{ output-schemata-path }}
    cargo run --package openfx-datagen --bin cli -- gen-schemata --output-schemata "{{ output-schemata-path }}"

detect-stale-generated-schemata: generate-schemata
    #!/usr/bin/env sh
    if ! git diff --quiet; then
        echo "Stale generated schemata detected."
        echo "Please run \`just generate-all\` and commit the changes."
        echo "stale files:"
        git --no-pager diff --name-only
        exit 1
    fi

generate-reference-bindings:
    rm -rf test-fixtures/reference-bindings/generated
    mkdir -p test-fixtures/reference-bindings/generated
    cd test-fixtures/reference-bindings && cargo xtask generate-reference-bindings \
        --wrapper ./wrapper.h --submodule ../openfx --output ./generated/bindings.rs

detect-stale-generated-reference-bindings:
    cd test-fixtures/reference-bindings && cargo xtask detect-stale-generated-reference-bindings \
        --wrapper ./wrapper.h --submodule ../openfx --bindings ./generated/bindings.rs

compare-generated-bindings-against-reference-bindings:
    rm -rf test-fixtures/tmp/our-bindings test-fixtures/tmp/data
    mkdir -p test-fixtures/tmp/our-bindings test-fixtures/tmp/data

    cargo run --package openfx-datagen --bin cli -- gen-data \
        --input-c-headers ./test-fixtures/openfx/include \
        --output-bindings-data ./test-fixtures/tmp/data/bindings

    cargo run --package openfx-bindgen --bin cli -- \
        --input-data ./test-fixtures/tmp/data/bindings \
        --output ./test-fixtures/tmp/our-bindings

    cargo xtask compare-generated-bindings \
        --config ./test-fixtures/reference-bindings/comparison.toml \
        --openfx-bindgen-bindings-folder ./test-fixtures/tmp/our-bindings \
        --reference-bindings-file test-fixtures/reference-bindings/generated/bindings.rs
