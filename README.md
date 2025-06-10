# alertrs

The server for alert

## Development

### Requirements

- [PostgreSQL](https://www.postgresql.org/)
- [Direnv](https://direnv.net/)
- [SQLX CLI](https://crates.io/crates/sqlx-cli)

## Building

### Setting up the database

Create the role and alert table for your Postgres instance. Here's the sql:

```
CREATE ROLE <db username> WITH LOGIN SUPERUSER <db password>;
CREATE DATABASE alert;
```
#### Environment variables

Environment variables are controlled via Direnv. You can either:

- Copy the example file
- Create one from scratch

##### Copy example

Copy the `.envrc_example` like this:

```
cd <cloned project directory>
cp .envrc_example .envrc
direnv allow .
```

##### Create one from scratch

Create a `.envrc` and add:

```
export DATABASE_URL=postgresql://<db username>:<db user password>@localhost:5432/sidekick
```

### Migrating the database

In the terminal run:

```
cd <cloned project directory>
direnv allow .
sqlx migrate run
```

### Running the server

In the terminal run:

```
cargo run
```

Server is available on: `localhost:8000`
