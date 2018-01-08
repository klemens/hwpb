# Development of hwpb

## Compile

See [`DEPLOY.md`] for details about setting up Rust. Then `cargo build` can be
used to build hwpb. If you just want to check that your changes compile, you
can use `cargo check`, which skips code generation and is therefore usually
much faster.

## Configure and run

Before running hwpb, you need to set up a PostgreSQL database and user and
add its credentials to a `Rocket.toml` configuration file in the top directory
of the project. You should also add your username to the `site_admins` list
to be able to log in (initially there are no others users in the database).

```toml
[global]
database = "postgres://user:password@host/database"
site_admins = [ "username" ]
```

You can optionally add a `secret_key` like described in [`DEPLOY.md`], if you
don't want log in every time after restarting hwpb. Otherwise the key will
be generated automatically on startup. Note that authenticating using PAM as
your own local user always works, even when you cannot read `/etc/shadow`.

After this, you should be able to run hwpb using `cargo run`. This will also
compile the project if there are any changes.

## Application overview

### Database

hwpb uses `Diesel` as a query builder and ORM to access a PostgreSQL database.

The `db` module contains code to setup the database pool and integration with
`Rocket` to get a connection from the pool for every request. The `db::schema`
module defines the database schema, which is used to typecheck all queries,
so generally no invalid queries can be compiled. The schema definitions and
posible joins between tables can be generated automatically from the schema
in the database using the [`diesel` cli] tool.

Joins are mostly automatic from the foreign key constraints of the schema, but
you can also join non-related tables by using the `enable_multi_table_joins!`
macro and manually specifying the ON-clause.

The `db::models` module contains struct definitions that are used to query or
insert data. They use regular Rust types, which are converted to the right
SQL types by Diesel automatically. To use these structs in the query builder,
they have to implement different traits (interfaces) like `Queryable`,
`Insertable` or `Identifiable`. Most of them can be derived (implemented)
automatically, but some require additional annotations. Have a look at the
[Diesel documentation] for more details.

To change the database schema over time, Diesel supports migrations. A single
migration always consists of one file that applies the changes (`up.sql`) and
one that undoes them again (`down.sql`). All migrations are embedded into the
binary during compilation and the ones that have not been applied yet to the
database are applied on the next application startup. You can also use the
[`diesel` cli] tool to manage migrations.

### HTTP interface

hwpb uses `Rocket` for delivering static files and providing its REST-API.

Every route is represented as a simple function annotated with the request
method and its path. To access information about the request, Rocket uses
simple function parameters (request guards) with regular Rust types. A request
is only handed to a specific function, if the path is correct and all
parameters are present and parse successfully, e.g. by using the `FromParam`
trait.

A similar trait (`FromRequest`) is used in `web::session` to access the cookies
of a request. Our implementation of this trait for the `User` struct only
succeeds if the user is logged in, so `User` can be used as a request guard to
restrict access of a route to logged in users (note that you must additionally
check if the user is allowed to view the current site or execute the current
action using for example `User::is_tutor_for()`). Similarly, this mechanism can
be used to request access to local resources like a database connection using
the `db::Conn` struct.

For more details, have a look at the excellent [Rocket guide] and the
[Rocket documentation].

### Templates

hwpb uses `Tera` for rendering templates. The syntax is very similar to the one
used in Jinja2.

All templates extend a base template that defines the basic HTML structure.
The child templates then override specific blocks in the base template to add
their functionality.

## Updating dependencies

cargo uses two files for specifying dependencies. `Cargo.toml` specifies the
direct requirements in a semantic versioning style, while `Cargo.lock` contains
the actual versions of all (recursive) requirements and is generated
automatically when building the project.

`cargo update` can be used to update `Cargo.lock` to the newest compatible
versions (so generally this should not require any changes to your own code),
while updating to a new major version of a dependency requires updating
`Cargo.toml` (and then running `cargo update` afterwards) manually.

[`DEPLOY.md`]: DEPLOY.md
[`diesel` cli]: https://github.com/diesel-rs/diesel/tree/master/diesel_cli
[Diesel documentation]: http://docs.diesel.rs/
[Rocket guide]: https://rocket.rs/guide/
[Rocket documentation]: https://api.rocket.rs/rocket/
