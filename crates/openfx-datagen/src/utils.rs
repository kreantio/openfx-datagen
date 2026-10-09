pub fn write_schema_json_pretty<W: std::io::Write, T: ?Sized + schemars::JsonSchema>(
    writer: W,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schema_generator = schemars::generate::SchemaSettings::default()
        .with_transform(schemars::transform::RecursiveTransform(
            |s: &mut schemars::Schema| {
                s.remove("description");
            },
        ))
        .into_generator();

    let bindings_schema = schema_generator.into_root_schema_for::<T>();
    serde_json::to_writer_pretty(writer, &bindings_schema)?;
    Ok(())
}
