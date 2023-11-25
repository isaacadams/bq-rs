use std::marker::PhantomData;

pub struct ResponseThunk<T> {
    pub response: ureq::Response,
    item: PhantomData<T>,
}

impl<T> ResponseThunk<T> {
    pub fn new(response: ureq::Response) -> Self {
        Self {
            response,
            item: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn info(&self) -> String {
        format!(
            "{} {} {} {}",
            self.response.status(),
            self.response.status_text(),
            self.response.get_url(),
            self.response.http_version()
        )
    }
}

impl<T> ResponseThunk<T>
where
    for<'a> T: serde::de::Deserialize<'a>,
{
    pub fn deserialize(self) -> Result<T, serde_json::Error> {
        let raw = self.response.into_string().unwrap();
        serde_json::from_str(&raw)
    }
}
