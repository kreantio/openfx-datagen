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

pub struct SignificantLines<'a>(std::str::Lines<'a>);

impl<'a> SignificantLines<'a> {
    pub fn new(lines: std::str::Lines<'a>) -> Self {
        Self(lines)
    }
}

impl<'a> std::iter::Iterator for SignificantLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        for line in self.0.by_ref() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                return Some(line);
            }
        }
        None
    }
}
