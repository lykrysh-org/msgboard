use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use chrono::prelude::{NaiveDateTime};
use schema::{
    tasks,
    tasks::dsl::{
        rootnum as task_root,
        editable as task_editable,
        tasks as all_tasks,
    },
    secrets,
};

#[derive(Debug, Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    pub whosent: String,
    pub attached: Option<String>,
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
    pub rootnum: i32,
    pub replnum: i32,
    pub posted: NaiveDateTime,
    pub whosent: String,
    pub attached: Option<String>,
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

pub struct EditTask {
    pub id: i32,
    pub linky: Option<String>,
    pub desc: String,
    pub sameimg: bool,
}

impl Task {
    pub fn all(conn: &PgConnection) -> QueryResult<Vec<Task>> {
        use schema::tasks::dsl::*;
        tasks
            .order((rootnum.desc(), replnum.asc()))
            .load::<Task>(conn)
    }

    pub fn inserttask(todo: NewTask, conn: &PgConnection) -> i32 {
        let row_inserted = diesel::insert_into(tasks::table)
            .values(&todo)
            .get_result::<Task>(conn)
            .unwrap();
        return row_inserted.id
    }

    pub fn set_as_root(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        let updated_task = diesel::update(all_tasks.find(idd));
        updated_task
            .set(task_root.eq(idd))
            .execute(conn)
    }

    pub fn get_max_replnum(parentid: i32, conn: &PgConnection) -> QueryResult<i32> {
        use schema::tasks::dsl::*;
        tasks
            .filter(rootnum.eq(parentid))
            .select(replnum)
            .order(replnum.desc())
            .first::<i32>(conn)
    }

    pub fn set_as_repl(idd: i32, parentid: i32, repl: i32, conn: &PgConnection) -> QueryResult<usize> {
        use schema::tasks::dsl::*;
        let updated_task = diesel::update(tasks.find(idd));
        updated_task
            .set((rootnum.eq(parentid), replnum.eq(repl)))
            .execute(conn)
    }

    pub fn insertsecret(key: NewSecret, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(secrets::table)
            .values(&key)
            .execute(conn)
    }

    pub fn get_secret(idd: i32, conn: &PgConnection) -> QueryResult<String> {
        use schema::secrets::dsl::*;
        secrets
            .filter(taskid.eq(idd))
            .select(secret)
            .first::<String>(conn)
    }

    pub fn toggle_with_id(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        let task = all_tasks.find(idd)
            .get_result::<Task>(conn)?;
        let new_status = !task.editable;
        let updated_task = diesel::update(all_tasks.find(idd));
        updated_task
            .set(task_editable.eq(new_status))
            .execute(conn)
    }

    pub fn delete_with_id(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(all_tasks.find(idd))
            .execute(conn)
    }

    pub fn re_write_desc(t: &EditTask, conn: &PgConnection) -> QueryResult<usize> {
        use schema::tasks::dsl::*;
        let updated_task = diesel::update(tasks.find(t.id));
        if t.sameimg {
            updated_task
                .set(description.eq(t.desc.clone()))
                .execute(conn)
        } else {
            updated_task
                .set((description.eq(t.desc.clone()), attached.eq(t.linky.clone())))
                .execute(conn)
        }
    }
}

