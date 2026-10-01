output-schemata-path := "schemata/generated"

generate-schemata:
    rm -rf {{ output-schemata-path }}
    mkdir -p {{ output-schemata-path }}
    cargo run --package openfx-datagen --bin cli -- gen-schemata --output-schemata "{{ output-schemata-path }}"
