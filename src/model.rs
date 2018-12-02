use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use chrono::prelude::{NaiveDateTime};

use schema::{
    tasks, tasks::dsl::{editable as task_editable, description as task_desc, tasks as all_tasks},
    secrets, secrets::dsl::{taskid as secret_taskid, secret as secret_secret, secrets as all_secrets},
};

#[derive(Debug, Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    pub whosent: String,
    pub description: String,
}

#[derive(Debug, Insertable)]
#[table_name = "secrets"]
pub struct NewSecret {
    pub secret: String,
    pub taskid: i32,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Task {
    pub id: i32,
    pub posted: NaiveDateTime,
    pub whosent: String,
    pub editable: bool,
    pub description: String,
}

#[derive(Debug, Queryable, Serialize, Associations, PartialEq)]
#[belongs_to(Task, foreign_key = "taskid")]
pub struct Secret {
    pub id: i32,
    pub secret: String,
    pub taskid: i32,
}

impl Task {
    pub fn all(conn: &PgConnection) -> QueryResult<Vec<Task>> {
        all_tasks
            .order(tasks::id.desc())
            .load::<Task>(conn)
    }

    pub fn inserttask(todo: NewTask, conn: &PgConnection) -> i32 {
        let row_inserted = diesel::insert_into(tasks::table)
            .values(&todo)
            .get_result::<Task>(conn)
            .unwrap();
        return row_inserted.id
    }

    pub fn insertsecret(key: NewSecret, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(secrets::table)
            .values(&key)
            .execute(conn)
    }

    pub fn get_secret(id: i32, conn: &PgConnection) -> QueryResult<String> {
        all_secrets
            .filter(secret_taskid.eq(id))
            .select(secret_secret)
            .first::<String>(conn)
    }

    pub fn toggle_with_id(id: i32, conn: &PgConnection) -> QueryResult<usize> {
        let task = all_tasks.find(id)
            .get_result::<Task>(conn)?;
        let new_status = !task.editable;
        let updated_task = diesel::update(all_tasks.find(id));
        updated_task
            .set(task_editable.eq(new_status))
            .execute(conn)
    }

    pub fn delete_with_id(id: i32, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(all_tasks.find(id))
            .execute(conn)
    }

    pub fn re_write_desc(id: i32, _desc: String, conn: &PgConnection) -> QueryResult<usize> {
        let updated_task = diesel::update(all_tasks.find(id));
        updated_task
            .set(task_desc.eq(_desc))
            .execute(conn)
    }

}

