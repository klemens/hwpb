use rocket::{Outcome, State};
use rocket::http::{Cookie, Cookies};
use rocket::http::uri::URI;
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
        let user = request.cookies()
            .get_private("username")
            .map(|cookie| User {
                name: cookie.value().into()
            });

        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Forward(())
        }
    }
}

pub struct NotLoggedIn;

impl<'a, 'r> FromRequest<'a, 'r> for NotLoggedIn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<NotLoggedIn, ()> {
        let user = request.cookies()
            .get_private("username");

        if user.is_none() {
            Outcome::Success(NotLoggedIn)
        } else {
            Outcome::Forward(())
        }
    }
}

#[get("/", rank = 2)]
fn nologin_index() -> Redirect {
    redirect_to_login("/")
}

#[get("/<_path..>", rank = 3)]
fn nologin_path(uri: &URI, _path: PathBuf, _user: NotLoggedIn) -> Redirect {
    redirect_to_login(uri.as_str())
}

#[get("/login")]
fn login_redirect(_user: User) -> Redirect {
    Redirect::to("/")
}

#[derive(FromForm)]
struct LoginOptions {
    redirect: String,
}

#[get("/login?<options>")]
fn get_login(options: LoginOptions, error: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    context.insert("redirect", options.redirect.as_str());
    if let Some(ref error) = error {
        context.insert("error", error.msg());
    }

    Template::render("login", &context)
}

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
    redirect: String,
}

#[post("/login", data = "<login>")]
fn post_login(mut cookies: Cookies, login: Form<LoginForm>, allowed_users: State<AllowedUsers>) -> Result<Redirect, Flash<Redirect>> {
    let login = login.into_inner();
    let redirect = URI::percent_decode_lossy(login.redirect.as_bytes());

    if !allowed_users.0.contains(&login.username) {
        let msg = "Ungültiger Benutzername!";
        return Err(Flash::error(redirect_to_login(&redirect), msg))
    }

    let result = user::authenticate(&login.username, &login.password);
    if result == Ok(true) {
        cookies.add_private(Cookie::new("username", login.username));
        Ok(Redirect::to(&redirect))
    } else {
        let msg = "Ungültiger Benutzername oder Passwort!";
        Err(Flash::error(redirect_to_login(&redirect), msg))
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("username"));
    Redirect::to("/")
}

fn redirect_to_login(sucess_redirect: &str) -> Redirect {
    let success_redirect = URI::percent_encode(sucess_redirect)
        .replace("&", "%26"); // '&' is not encoded by default like '?'
    Redirect::to(&format!("/login?redirect={}", success_redirect))
}
