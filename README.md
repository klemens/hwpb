# hwpb

Manage students' progress during the computer science hardware course at
Leipzig University.

## Build

The project can be built using cargo.

```sh
cargo build --release
```

Note that you currently need at least rust 1.24.0-nightly 2017-12-17.
Use [`rustup`] on windows or if the packages of your distribution are too old.

## Usage

hwpb expects a PostgreSQL database and a `Rocket.toml` configuration file in
the current working directory, which includes the database connection URI and
a list of initial site administrators. Authentication is done using PAM,
so make sure `/etc/shadow` is readable for your user or PAM is using a remote
authentication mechanism.

```sh
./target/release/hwpb
```

See the [`doc/DEPLOY.md`] and [`doc/DEVELOP.md`] files for more details.

[`rustup`]: https://www.rustup.rs/
[`doc/DEPLOY.md`]: doc/DEPLOY.md
[`doc/DEVELOP.md`]: doc/DEVELOP.md
