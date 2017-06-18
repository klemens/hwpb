use rocket::{Outcome, State};
use rocket::http::{Cookie, Session};
use rocket::request::{self, FlashMessage, Form, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use user;

pub struct AllowedUsers(HashSet<String>);

impl AllowedUsers {
    pub fn new<'a, I: IntoIterator<Item = &'a str>>(users: I) -> Self {
        AllowedUsers(users.into_iter().map(|s| s.to_string()).collect())
    }
}

pub struct User {
    pub name: String
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let user = request.session()
            .get("username")
            .map(|cookie| User {
                name: cookie.value().into()
            });

        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Forward(())
        }
    }
}

#[get("/", rank = 2)]
fn nologin_index() -> Redirect {
    Redirect::to("/login")
}

#[get("/<path..>", rank = 2)]
#[allow(unused_variables)]
fn nologin_path(path: PathBuf) -> Redirect {
    Redirect::to("/login")
}

#[get("/login")]
fn get_login(error: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref error) = error {
        context.insert("error", error.msg());
    }

    Template::render("login", &context)
}

#[derive(FromForm)]
struct Login {
    username: String,
    password: String
}

#[post("/login", data = "<login>")]
fn post_login(mut session: Session, login: Form<Login>, allowed_users: State<AllowedUsers>) -> Result<Redirect, Flash<Redirect>> {
    let login = login.into_inner();

    if !allowed_users.0.contains(&login.username) {
        let msg = "Ungültiger Benutzername!";
        return Err(Flash::error(Redirect::to("/login"), msg))
    }

    let result = user::authenticate(&login.username, &login.password);
    if result == Ok(true) {
        session.set(Cookie::new("username", login.username));
        Ok(Redirect::to("/"))
    } else {
        let msg = "Ungültiger Benutzername oder Passwort!";
        Err(Flash::error(Redirect::to("/login"), msg))
    }
}

#[get("/logout")]
fn logout(mut session: Session) -> Redirect {
    session.remove(Cookie::named("username"));
    Redirect::to("/")
}
