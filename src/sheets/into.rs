pub trait MyInto<T> {
    fn my_into(self) -> T;
}

impl MyInto<Vec<String>> for Vec<Vec<serde_json::Value>> {
    fn my_into(self) -> Vec<String> {
        self.into_iter()
            .flatten()
            .map(|v| v.to_string().replace('\"', ""))
            .collect::<Vec<String>>()
    }
}
