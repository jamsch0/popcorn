table! {
    films (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        title -> Text,
        release_year -> Int4,
        summary -> Text,
        runtime_mins -> Int4,
    }
}
