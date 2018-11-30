table! {
    tasks (id) {
        id -> Int4,
        posted -> Timestamptz,
        completed -> Bool,
        description -> Varchar,
    }
}
