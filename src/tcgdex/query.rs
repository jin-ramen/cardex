pub struct Query {
    param: String,
    arg: String,
}

impl Query {
    pub fn new(param: impl Into<String>, arg: impl Into<String>) -> Self {
        Self { param: param.into(), arg: arg.into() }
    }

    pub fn eq(param: impl Into<String>, arg: &str) -> Self {
        Self::new(param, format!("eq:{arg}"))
    }

    pub fn as_pair(&self) -> (&str, &str) {
        (&self.param, &self.arg)
    }
}