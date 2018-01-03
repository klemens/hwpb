# Deployment of hwpb

## Compile

hwpb is written in Rust and can be compiled using its package manager `cargo`:

```sh
cargo build --release
```

Note that you need a recent nightly version of the compiler, because we depend
on the web framework `Rocket`, which requires some unstable features that are
not available in a stable or beta version of Rust. The exact version necessary
is noted in the [`README.md`] and may change when updating `Rocket`.

If you are using `rustup` to manage your Rust installation, you can override
the used compiler version only for this project (i.e. the current directory):

```sh
$ rustup override set nightly-2017-12-21
$ rustc --version
rustc 1.24.0-nightly (7eb64b86c 2017-12-20)
```

## Configure

hwpb is configured through a `Rocket.toml` file and environment variables.
You can use the provided [example config] as a starting point. The settings
in the `production` section only take effect when the production environment
is enabled by setting the environment variable `ROCKET_ENV` to `production`.
Other possible values are `staging` and the default `development`. See the
[rocket documentation] for more details.

The `secret_key` is used to authenticate and encrypt cookies and must be kept
private. It can be generated using `openssl rand -base64 32`.

The `global` section contains setting that apply to all environments. The
`database` key specifies the URI that is used to connect to a PostgreSQL
database and `allowed_users` contains a list of users that are allowed to
log in and use the application.

The database is initialised automatically when running hwpb for the first time.

## Deploy

Besides the `Rocket.toml` configuration file, hwpb expects its templates in a
`templates` folder in the current working directory by default. This path can
be changed by setting the `template_dir` parameter in the config file.

Authentication is handled by the PAM library, so for local users to be able to
log in, the user running hwpb must have read access to `/etc/shadow` to check
passwords. A better alternative is to use a remote authentication plugin for
PAM like SSS, NIS or LDAP, which works without access to `/etc/shadow`.

It is advisable to run hwpb as a non-privileged user using a service manager
and letting a web server handle client connections and encryption. You can use
the provided [example systemd service] and the [example nginx config] as a
starting point. The former expects the binary, the `templates` folder and the
`Rocket.toml` configuration file in `/srv/hwpb`, which is owned by a user
`hwpb`.

[`README.md`]: ../README.md
[example config]: examples/Rocket.toml
[rocket documentation]: https://api.rocket.rs/rocket/config/
[example systemd service]: examples/hwpb.service
[example nginx config]: examples/hwbp.nginx
