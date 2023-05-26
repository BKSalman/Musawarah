# مُصَوّرَة

منصة للقصص المصورة

# Built with
This platform is developed using the following tools and technologies:

Backend:

- [Axum](https://github.com/tokio-rs/axum) Web framework
- [Garde](https://github.com/jprochazk/garde) Validation crate
- [Diesel](https://github.com/diesel-rs/diesel) Database ORM
- [TS-rs](https://github.com/Aleph-Alpha/ts-rs) Generate TS bindings from the backend models

Frontend:

- [Svelte-kit](https://kit.svelte.dev/) Meta webframework
- [sveltekit-superforms](https://github.com/ciscoheat/sveltekit-superforms) Useful tools for Sveltekit forms

# Building/Running the project
if you are on Linux/MacOS/WSL you can use the flake.nix file that contains the full dev environment:

just install [the nix package manager](https://zero-to-nix.com/start/install) and run:
```bash
# in project root
nix develop
```
### Backend
make sure you have [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) & [diesel-cli](https://crates.io/crates/diesel_cli) & [docker](https://www.docker.com/) installed (already done if using nix)
#### Database
to setup a dev database run the following commands:

> this will run a local docker postgresql database that you can develop on

(already done if using nix)
```bash
docker run --name musawarah-dev -p 5432:5432 -e POSTGRES_PASSWORD=musawarah-dev -d postgres
```
then create a `.env` file in the project root with the following line:
```bash
DATABASE_URL=postgres://postgres:musawarah-dev@localhost:5432
```
there are more environment variables needed, for those you can talk to Salman to give it to you :)

after you have all environment variables, you need to export them all in bash you do:
```bash
# in project root
export $(cat .env)
```

> this will use the migrations in the `migrations` folder to apply to the database
```bash
# first install diesel-cli with cargo (already installed if using nix)
cargo install diesel-cli

# in project root
# this will apply the migrations to the database
diesel-cli migration run
```

#### Run dev server
```bash
# in project root
cargo run
```
#### Run dev server with logging
```bash
# in project root
RUST_LOG=debug cargo run # unix-like shells only
```
#### Run tests & generate TS bindings/types
```bash
# in project root
cargo test
```

### Frontend
[frontend setup instructions](https://github.com/BKSalman/rmusawarah/blob/main/client/README.md)

# Backend Endpoints
All endpoints are documented with ``OpenAPI`` documentation standard, and can be viewed in ``Swagger`` by opening ``<baseurl>/swagger-ui/`` while the server is running
