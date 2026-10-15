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

pub struct LinesEx<'a> {
    text: &'a str,
    offset: usize,
}

pub struct LineEx<'a> {
    pub content: &'a str,
    pub start_offset: usize,
}

impl<'a> LinesEx<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            text: content,
            offset: 0,
        }
    }
}

impl<'a> Iterator for LinesEx<'a> {
    type Item = LineEx<'a>;

    /// Author: GitHub Copilot's tab completion
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.text.len() {
            return None;
        }
        let remaining_text = &self.text[self.offset..];
        if let Some(pos) = remaining_text.find('\n') {
            let mut line = &remaining_text[..pos];
            line = line.strip_suffix('\r').unwrap_or(line);
            let start_offset = self.offset;
            self.offset += pos + 1;
            Some(LineEx {
                content: line,
                start_offset,
            })
        } else {
            let start_offset = self.offset;
            self.offset = self.text.len();
            Some(LineEx {
                content: remaining_text,
                start_offset,
            })
        }
    }
}
