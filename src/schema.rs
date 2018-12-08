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
        rootnum -> Int4,
        replnum -> Int4,
        posted -> Timestamptz,
        whosent -> Varchar,
        attached -> Nullable<Varchar>,
        editable -> Bool,
        description -> Varchar,
    }
}

joinable!(secrets -> tasks (taskid));

allow_tables_to_appear_in_same_query!(
    secrets,
    tasks,
);
