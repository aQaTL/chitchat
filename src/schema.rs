table! {
    images (id) {
        id -> Int8,
        filename -> Varchar,
        creation_date -> Timestamp,
        content -> Bytea,
    }
}

table! {
    pastes (id) {
        id -> Int8,
        filename -> Nullable<Varchar>,
        content -> Nullable<Text>,
        creation_date -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    images,
    pastes,
);
