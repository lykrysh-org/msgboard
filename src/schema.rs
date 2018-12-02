table! {
    secrets (id) {
        id -> Int4,
        secret -> Varchar,
        taskid -> Int4,
    }
}

table! {
    tasks (id) {
        id -> Int4,
        posted -> Timestamptz,
        whosent -> Varchar,
        editable -> Bool,
        description -> Varchar,
    }
}

joinable!(secrets -> tasks (taskid));

allow_tables_to_appear_in_same_query!(
    secrets,
    tasks,
);
