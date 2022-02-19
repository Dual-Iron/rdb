use mongodb::bson::doc;

macro_rules! index {
    ($collection:expr, $doc:expr) => {
        $collection
            .create_index(mongodb::IndexModel::builder().keys($doc).build(), None)
            .await
            .expect("failed to create an index");
    };
}

pub(crate) async fn add_indexes<T>(collection: &mongodb::Collection<T>) {
    index!(collection, doc! { "search": "text" });
    index!(collection, doc! { "downloads": 1 });
    index!(collection, doc! { "updated": 1 });
}
