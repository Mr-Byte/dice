#[derive(Debug, Clone)]
pub struct Tags(pub(super) Vec<(&'static str, String)>);

impl Tags {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, key: &'static str, value: impl Into<String>) {
        self.0.push((key, value.into()));
    }
}

#[macro_export]
macro_rules! tags {
    ($($tag:ident => $value:expr),*) => {{
        let mut tags = $crate::error::tag::Tags::new();

        $(
            tags.push(stringify!($tag), $value);
        )*

        tags
    }}
}
