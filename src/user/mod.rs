use pam::Authenticator;

pub fn authenticate(username: &str, password: &str) -> Result<bool, ()> {
    let service = env!("CARGO_PKG_NAME");

    let mut auth = Authenticator::with_password(service).map_err(|_| ())?;
    auth.get_handler().set_credentials(username, password);

    auth.authenticate()
        .map(|_| true)
        .map_err(|_| ())
}
