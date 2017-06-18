use pam_auth::Authenticator;

pub fn authenticate(username: &str, password: &str) -> Result<bool, ()> {
    let service = env!("CARGO_PKG_NAME");

    let mut auth = Authenticator::new(service).ok_or(())?;
    auth.set_credentials(username, password);

    auth.authenticate()
        .map(|_| true)
        .map_err(|_| ())
}
