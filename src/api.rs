use actix::prelude::Addr;
use actix_web::middleware::Response;
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{
    fs::NamedFile, http, AsyncResponder, Form, FutureResponse, HttpRequest,
    HttpResponse, Path, Responder, Result, HttpMessage, Json, Error, 
};
use actix_web::error::ErrorInternalServerError;
use futures::{future, Future, Stream};
use tera::{Context, Tera};

use db::{AllTasks, CreateTask, DbExecutor, DeleteTask, ToggleTask, FindTask };
use model::{EditTask};
use session::{self, FlashMessage, UpLoaded, };
use multipart::*;

pub struct AppState {
    pub template: Tera,
    pub db: Addr<DbExecutor>,
}

pub fn index(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(AllTasks)
        .from_err()
        .and_then(move |res| match res {
            Ok(tasks) => {
                let mut context = Context::new();
                context.insert("tasks", &tasks);

                //Sessions
                if let Some(flash) = session::get_flash(&req)? {
                    context.insert("msg", &(flash.kind, flash.message));
                    session::clear_flash(&req);
                }

                //Identity
                if let Some(id) = req.identity() {
                    context.insert("welcomeback", &id)
                }

                let rendered = req.state()
                    .template
                    .render("index.tera", &context)
                    .map_err(|e| {
                        ErrorInternalServerError(e.description().to_owned())
                    })?;

                Ok(HttpResponse::Ok().body(rendered))
            }
            Err(e) => Err(e),
        })
        .responder()
}

pub fn multipart(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    println!("{:?}", req);
    Box::new(
        req.multipart()
            .map_err(ErrorInternalServerError)
            .map(handle_multipart_item)
            .flatten()
            .collect()
            .map(move |name| {
                println!("{}", &name[0]);
                let _ = session::set_uploaded(
                    &req,
                    UpLoaded::add(&name[0]),
                );
                HttpResponse::Ok().finish()
            })
            .map_err(|e| {
                println!("failed multipart: {}", e);
                e
            }),
    )
}

#[derive(Debug, Deserialize)]
pub struct CreateJ {
    inheritedid: String,
    hasimg: String,
    secret: String,
    whosent: String,
    linky: String,
    description: String,
}

#[derive(Serialize)]
struct OutJ {
    state: String,
}

pub fn create(
    (req, j): (HttpRequest<AppState>, Json<CreateJ>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    println!("{:?} {:?}", req, j );

    let lnk: Option<String> = match j.hasimg.parse().unwrap_or(0) {
        1 => {
                let up = match session::get_uploaded(&req).unwrap() {
                    Some(up) => Some(up.uploaded),
                    None => None
                };
                session::clear_uploaded(&req);
                up           
             },
        2 => Some(j.linky.clone()),
        _ => None,
    };

    let replyid: Option<i32> = match j.inheritedid.clone().as_ref() {
        "none" => None,
        _whatever => {
            let i: i32 = _whatever.parse().unwrap_or(0);
            Some(i)
        },
    };
    let name = j.whosent.clone();
    req.state()
        .db
        .send(CreateTask {
            inheritedid: replyid,
            secret: j.secret.clone(),
            whosent: name.to_string(),
            linky: lnk,
            description: j.description.clone().trim().to_string(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(taskid) => {
                let out = OutJ {
                    state: taskid.to_owned().to_string(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Debug, Deserialize)]
pub struct PassdJ {
    taskid: String,
    method: String,
    passwd: String,
}

pub fn passd(
    (req, j) : (HttpRequest<AppState>, Json<PassdJ>),
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    println!("{:?} {:?}", req, j );
    let id = j.taskid.parse::<i32>().unwrap();
    match j.method.as_ref() {
        "put" => Box::new(toggle(req, &id, &j.passwd)),
        "delete" => Box::new(delete(req, &id, &j.passwd)),
        _ => Box::new(future::ok(HttpResponse::Ok().finish())),
    }
}

fn toggle(
    req: HttpRequest<AppState>,
    id: &i32,
    mypw: &String,
) -> impl Future<Item = HttpResponse, Error = Error> {
    req.state()
        .db
        .send(ToggleTask { id: *id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(0) => {
                let out = OutJ {
                    state: "wrong".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Ok(_taskid) => {
                let out = OutJ {
                    state: "correct".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Err(e) => Err(e),
        })
        .responder()
}

fn delete(
    req: HttpRequest<AppState>,
    id: &i32,
    mypw: &String,
) -> impl Future<Item = HttpResponse, Error = Error> {
    req.state()
        .db
        .send(DeleteTask { id: *id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(0) => {
                let out = OutJ {
                    state: "wrong".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Ok(_) => {
                let out = OutJ {
                    state: "deleted".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct UpdateParams {
    id: i32,
}

#[derive(Debug, Deserialize)]
pub struct EditJ {
    hasimg: String,
    linky: String,
    description: String,
}

pub fn edit(
    (req, params, j): (HttpRequest<AppState>, Path<UpdateParams>, Json<EditJ>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    println!("{:?} {:?}", req, j );

    let hinum = j.hasimg.parse().unwrap_or(0);
    let lnk: Option<String> = match hinum {
        1 => {
                let up = match session::get_uploaded(&req).unwrap() {
                    Some(up) => Some(up.uploaded),
                    None => None
                };
                session::clear_uploaded(&req);
                up
             },
        2 => Some(j.linky.clone()),
        _ => None,
    };
    if hinum == 4 {
        req.state()
            .db
            .send(EditTask {
                id: params.id,
                linky: lnk,
                desc: j.description.trim().to_string(),
                sameimg: true,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    let out = OutJ {
                        state: "edited except img".to_owned(),
                    };
                    let o = serde_json::to_string(&out)?;
                    Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                },
                Err(e) => Err(e),
            })
            .responder()
    } else {
        req.state()
            .db
            .send(EditTask {
                id: params.id,
                linky: lnk,
                desc: j.description.trim().to_string(),
                sameimg: false,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    let out = OutJ {
                        state: "edited including img".to_owned(),
                    };
                    let o = serde_json::to_string(&out)?;
                    Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                },
                Err(e) => Err(e),
            })
            .responder()
    }
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, location)
        .finish()
}

pub fn bad_request<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("static/errors/400.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

pub fn not_found<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("static/errors/404.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

pub fn internal_server_error<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("static/errors/500.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

