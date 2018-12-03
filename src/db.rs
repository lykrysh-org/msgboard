use std::ops::Deref;

use actix::prelude::{Actor, Handler, Message, SyncContext};
use actix_web::{error, Error};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use model::{NewTask, Task, NewSecret};

type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub struct DbExecutor(pub PgPool);

impl DbExecutor {
    pub fn get_conn(&self) -> Result<PgPooledConnection, Error> {
        self.0.get().map_err(|e| error::ErrorInternalServerError(e))
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct AllTasks;

impl Message for AllTasks {
    type Result = Result<Vec<Task>, Error>;
}

impl Handler<AllTasks> for DbExecutor {
    type Result = Result<Vec<Task>, Error>;

    fn handle(&mut self, _: AllTasks, _: &mut Self::Context) -> Self::Result {
        Task::all(self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error inserting task"))
    }
}

pub struct CreateTask {
    pub inheritedid: Option<i32>,
    pub secret: String,
    pub whosent: String,
    pub description: String,
}

impl Message for CreateTask {
    type Result = Result<(), Error>;
}

impl Handler<CreateTask> for DbExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, todo: CreateTask, _: &mut Self::Context) -> Self::Result {
        let new_task = NewTask {
            whosent: todo.whosent,
            description: todo.description,
        };
        let tid = Task::inserttask(new_task, self.get_conn()?.deref());
        let _ = match todo.inheritedid.as_ref() {
            Some(parentid) => {
                let replnum = Task::get_max_replnum(*parentid, self.get_conn()?.deref())
                    .map_err(|_| error::ErrorInternalServerError("Error get_max_replynum"));
                match replnum {
                    Ok(num) => {
                        let new: i32 = num + 1;
                        let _ = Task::set_as_repl(tid, *parentid, new, self.get_conn()?.deref())
                            .map(|_| ())
                            .map_err(|_| error::ErrorInternalServerError("Error set_as_repl"));
                    },
                    Err(_) => (),
                };
            },
            None => {
                let _ = Task::set_as_root(tid, self.get_conn()?.deref())
                    .map(|_| ())
                    .map_err(|_| error::ErrorInternalServerError("Error set_as_root"));
            }
        };
        let new_secret = NewSecret {
            secret: todo.secret,
            taskid: tid,
        };
        Task::insertsecret(new_secret, self.get_conn()?.deref())
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error inserting secret"))
    }
}

pub struct ToggleTask {
    pub id: i32,
    pub pw: String,
}

impl Message for ToggleTask {
    type Result = Result<usize, Error>;
}

impl Handler<ToggleTask> for DbExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, task: ToggleTask, _: &mut Self::Context) -> Self::Result {
        let pw = Task::get_secret(task.id, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error checking secret"));
        match pw {
            Ok(secret) => {
                if secret == task.pw {
                    Task::toggle_with_id(task.id, self.get_conn()?.deref())
                        .map_err(|_| error::ErrorInternalServerError("Error deleting task"))
                } else {
                    // wrong password
                    Ok(999)
                }
            },
            Err(e) => Err(e),
        }
    }
}

pub struct DeleteTask {
    pub id: i32,
    pub pw: String,
}

impl Message for DeleteTask {
    type Result = Result<usize, Error>;
}

impl Handler<DeleteTask> for DbExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, task: DeleteTask, _: &mut Self::Context) -> Self::Result {
        let pw = Task::get_secret(task.id, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error checking secret"));
        match pw {
            Ok(secret) => {
                if secret == task.pw {
                    Task::delete_with_id(task.id, self.get_conn()?.deref())
                        .map_err(|_| error::ErrorInternalServerError("Error deleting task"))
                } else {
                    // wrong password
                    Ok(999)
                }
            },
            Err(e) => Err(e),
        }
    }
}

pub struct UploadTask {
    pub id: i32,
    pub desc: String,
}

impl Message for UploadTask {
    type Result = Result<usize, Error>;
}

impl Handler<UploadTask> for DbExecutor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, task: UploadTask, _: &mut Self::Context) -> Self::Result {
        let _ = Task::re_write_desc(task.id, task.desc, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error deleting task"));
        Task::toggle_with_id(task.id, self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error deleting task"))
    }
}

pub struct CancelTask {
    pub id: i32,
}

impl Message for CancelTask {
    type Result = Result<(), Error>;
}

impl Handler<CancelTask> for DbExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, task: CancelTask, _: &mut Self::Context) -> Self::Result {
        Task::toggle_with_id(task.id, self.get_conn()?.deref())
            .map(|_| ())
            .map_err(|_| error::ErrorInternalServerError("Error deleting task"))
    }
}

