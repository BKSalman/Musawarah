
# WIP README

# مُصَوّرَة - Musawarah

A platform for artists to share their creative work

# Built with
This platform is developed using the following tools and technologies:
Back-end:

- [Axum](https://github.com/tokio-rs/axum) Web framework
- [garde](https://github.com/jprochazk/garde) Validation crate
- [sqlx](https://github.com/launchbadge/sqlx) SQL Database crate
- [ts-rs](https://github.com/Aleph-Alpha/ts-rs) Generate TS bindings from the backend models

Front-end:

- [Svelte-kit](https://kit.svelte.dev/) Meta webframework
- [sveltekit-superforms](https://github.com/ciscoheat/sveltekit-superforms) Useful tools for Sveltekit forms

# Endpoints
### All endpoints are documented with ``OpenAPI`` documentation standard, and can be viewed in ``Swagger``

and can be accessed by running the server with:
```
cargo run
```
then opening ``<baseurl>/swagger-ui/``


But here are the general docs for them:


### User endpoints

- Create user
```
POST /api/users

body example:
{
  "email": "string",
  "password": "string",
  "username": "string"
}
```


- User login
```
POST /api/users/login

body example:
{
  "email": "string",
  "password": "string"
}
```


- Get user posts by username
```
GET /api/users/{username}
```


### Posts endpoints

- Create post
```
POST /api/posts

multi-part form example:

"title": "string"
"content": "string"
"image": <file>
```


- Get user post by id
```
GET /api/posts/{username}/{post_id}
```
