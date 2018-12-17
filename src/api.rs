use actix::prelude::Addr;
use actix_web::middleware::Response;
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{
    error, fs::NamedFile, http, AsyncResponder, Form, FutureResponse, HttpRequest,
    HttpResponse, Path, Responder, Result, HttpMessage, Json,
};
use futures::{future, Future, Stream};
use tera::{Context, Tera};

use db::{AllTasks, CreateTask, DbExecutor, DeleteTask, ToggleTask };
use model::{EditTask};
use session::{self, FlashMessage, UpLoaded, PowerTo};
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
                if let Some(pt) = session::get_powerto(&req)? {
                    context.insert("powerto", &pt.powerto)
                }

                //Identity
                if let Some(id) = req.identity() {
                    context.insert("welcomeback", &id)
                }

                let rendered = req.state()
                    .template
                    .render("index.tera", &context)
                    .map_err(|e| {
                        error::ErrorInternalServerError(e.description().to_owned())
                    })?;

                Ok(HttpResponse::Ok().body(rendered))
            }
            Err(e) => Err(e),
        })
        .responder()
}

pub fn save(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    println!("{:?}", req);
    Box::new(
        req.multipart()
            .map_err(error::ErrorInternalServerError)
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

#[derive(Deserialize)]
pub struct CreateForm {
    inheritedid: String,
    hasimg: String,
    secret: String,
    whosent: String,
    linky: Option<String>,
    description: String,
}

pub fn create(
    (req, params): (HttpRequest<AppState>, Form<CreateForm>),
) -> FutureResponse<HttpResponse> {
    let lnk: Option<String> = match params.hasimg.parse().unwrap_or(0) {
        1 => {
                let up = match session::get_uploaded(&req).unwrap() {
                    Some(up) => Some(up.uploaded),
                    None => None
                };
                session::clear_uploaded(&req);
                up           
             },
        2 => params.linky.clone(),
        _ => None,
    };

    let replyid: Option<i32> = match params.inheritedid.clone().as_ref() {
        "none" => None,
        _whatever => {
            let i: i32 = _whatever.parse().unwrap_or(0);
            Some(i)
        },
    };
    let name = params.whosent.clone();
    req.state()
        .db
        .send(CreateTask {
            inheritedid: replyid,
            secret: params.secret.clone(),
            whosent: name.to_string(),
            linky: lnk,
            description: params.description.clone().trim().to_string(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                session::set_flash(
                    &req,
                    FlashMessage::success(&format!("Thanks {}!", name)),
                )?;
                &req.remember(name.to_owned());
                Ok(redirect_to("/"))
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

#[derive(Serialize)]
pub struct SendnJ {
    name: String,
}

pub fn passd(
    (req, j) : (HttpRequest<AppState>, Json<PassdJ>),
) -> FutureResponse<String> {
    println!("{:?} {:?}", req, j );
    let id = j.taskid.parse::<i32>().unwrap();
    match j.method.as_ref() {
        "put" => toggle(req, &id, &j.passwd),
        "delete" => delete(req, &id, &j.passwd),
        unsupported_method => {
            let msg = format!("Unsupported HTTP method: {}", unsupported_method);
            future::ok("hello".to_string()).responder()
        }
    }
}

fn toggle(
    req: HttpRequest<AppState>,
    id: &i32,
    mypw: &String,
) -> FutureResponse<String> {
    req.state()
        .db
        .send(ToggleTask { id: *id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(0) => {
                session::set_flash(&req, FlashMessage::error("Wrong password."))?;
                Ok("hello".to_string())
            },
            Ok(taskid) => {
                session::set_powerto(&req, PowerTo::add(taskid))?;
                Ok("hello".to_string())
            }
            Err(e) => Err(e),
        })
        .responder()
}

fn delete(
    req: HttpRequest<AppState>,
    id: &i32,
    mypw: &String,
) -> FutureResponse<String> {
    req.state()
        .db
        .send(DeleteTask { id: *id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(0) => {
                session::set_flash(&req, FlashMessage::error("Wrong password."))?;
                Ok("hello".to_string())
            },
            Ok(_) => {
                session::set_flash(&req, FlashMessage::success("Deleted."))?;
                Ok("hello".to_string())
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct UpdateParams {
    id: i32,
}

#[derive(Deserialize)]
pub struct EditForm {
    hasimg: String,
    linky: Option<String>,
    description: String,
}

pub fn edit(
    (req, params, form): (HttpRequest<AppState>, Path<UpdateParams>, Form<EditForm>),
) -> FutureResponse<HttpResponse> {
    let hinum = form.hasimg.parse().unwrap_or(0);
    let lnk: Option<String> = match hinum {
        1 => {
                let up = match session::get_uploaded(&req).unwrap() {
                    Some(up) => Some(up.uploaded),
                    None => None
                };
                session::clear_uploaded(&req);
                up
             },
        2 => form.linky.clone(),
        _ => None,
    };
    if hinum == 4 {
        req.state()
            .db
            .send(EditTask {
                id: params.id,
                linky: lnk,
                desc: form.description.trim().to_string(),
                sameimg: true,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    if let Some(_) = session::get_powerto(&req).unwrap() {
                        session::clear_powerto(&req)
                    };
                    Ok(redirect_to("/"))
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
                desc: form.description.trim().to_string(),
                sameimg: false,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    if let Some(_) = session::get_powerto(&req).unwrap() {
                        session::clear_powerto(&req)
                    };
                    Ok(redirect_to("/"))
                },
                Err(e) => Err(e),
            })
            .responder()
    }
}

pub fn cancel(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    if let Some(_) = session::get_powerto(&req).unwrap() {
        session::clear_powerto(&req)
    };
    future::ok(redirect_to("/"))
        .responder()
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

