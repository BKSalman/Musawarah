# مُصَوّرَة

منصة للقصص المصورة

# Built with
This platform is developed using the following tools and technologies:

Backend:

- [Axum](https://github.com/tokio-rs/axum) Web framework
- [garde](https://github.com/jprochazk/garde) Validation crate
- [sqlx](https://github.com/launchbadge/sqlx) SQL Database crate
- [ts-rs](https://github.com/Aleph-Alpha/ts-rs) Generate TS bindings from the backend models

Frontend:

- [Svelte-kit](https://kit.svelte.dev/) Meta webframework
- [sveltekit-superforms](https://github.com/ciscoheat/sveltekit-superforms) Useful tools for Sveltekit forms

# Building/Running the project
if you are on Linux/MacOS/WSL you can use the flake.nix file that contains the full dev environment:

just install [the nix package manager](https://zero-to-nix.com/start/install) and run:
```
# in project root
nix develop
```
### Backend
make sure you have [Rust](https://www.rust-lang.org/) & [sqlx-cli](https://crates.io/crates/sqlx-cli) & [docker](https://www.docker.com/) installed (already done if using nix)
#### Database
to setup a dev database run the following commands:

> this will run a local docker postgresql database that you can develop on

(already done if using nix)
```
docker run --name musawarah-dev -e POSTGRES_PASSWORD=musawarah-dev -d postgres -p 5432:5432
```
then create a `.env` file in the project root with the following line:
```
DATABASE_URL=postgres://postgres:musawarah-dev@localhost:5432
```
there are more environment variables needed, for those you can talk to Salman to give it to you :)

after you have all environment variables, you need to export them all in bash you do:
```bash
# in project root
export $(cat .env)
```

> this will use the migration details in the `migrations` folder to add to the database
```
# first install sqlx with cargo
cargo install sqlx-cli

# in project root
sqlx migrate run
```

#### Run dev server
```
# in project root
cargo run
```
#### Run dev server with logging
```
# in project root
RUST_LOG=debug cargo run
```
#### Run tests & generate TS bindings/types
```
# in project root
cargo test
```

### Frontend
[frontend setup instructions](https://github.com/BKSalman/rmusawarah/blob/main/client/README.md)

# Backend Endpoints
All endpoints are documented with ``OpenAPI`` documentation standard, and can be viewed in ``Swagger`` by opening ``<baseurl>/swagger-ui/`` while the server is running
