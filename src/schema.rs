table! {
    tasks (id) {
        id -> Int4,
        posted -> Timestamptz,
        whosent -> Varchar,
        secret -> Varchar,
        completed -> Bool,
        description -> Varchar,
    }
}
