use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct NameQuery {
    pub(crate) name: String,
}

impl From<String> for NameQuery {
    fn from(value: String) -> Self {
        NameQuery { name: value }
    }
}

#[derive(Deserialize)]
pub(crate) struct IndexQuery {
    pub(crate) index: usize,
}

impl From<usize> for IndexQuery {
    fn from(value: usize) -> Self {
        IndexQuery { index: value }
    }
}
