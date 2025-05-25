pub trait FlattenDoubleVec<T> {
    fn flatten_double_vec(self) -> T;
}

impl FlattenDoubleVec<Vec<String>> for Vec<Vec<serde_json::Value>> {
    fn flatten_double_vec(self) -> Vec<String> {
        self.into_iter()
            .flatten()
            .map(|v| v.to_string().replace('\"', ""))
            .collect::<Vec<String>>()
    }
}
