// Copyright 2019 James Chapman

#![feature(async_await, await_macro, futures_api)]
#![warn(clippy::all, rust_2018_idioms)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate tower_web;
#[macro_use]
extern crate juniper;

use std::env;

use diesel::prelude::*;
use tokio::prelude::*;

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use dotenv::dotenv;
use http::StatusCode;
use juniper::{FieldError, GraphQLType, RootNode};
use tower_web::ServiceBuilder;
use uuid::Uuid;

mod models;
mod schema;

use crate::models::{CreateFilm, Film};
use crate::schema::films;

pub struct Context {
    db_conn_pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl Context {
    pub fn db_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
        self.db_conn_pool.get()
    }
}

impl juniper::Context for Context {}

type Schema = RootNode<'static, Query, Mutation>;

struct Query;

graphql_object!(Query: Context |&self| {
    field get_films(&executor, first: Option<i32>, offset: Option<i32>) -> Result<Vec<Film>, FieldError> {
        let db = executor.context().db_conn()?;

        let mut query = films::table.into_boxed();
        if let Some(first) = first {
            query = query.limit(first.into());
        }
        if let Some(offset) = offset {
            query = query.offset(offset.into());
        }

        let films = query.load(&db)?;

        Ok(films)
    }

    field get_film(&executor, id: Uuid) -> Result<Option<Film>, FieldError> {
        let db = executor.context().db_conn()?;
        let film = films::table.find(id)
            .first(&db)
            .ok();

        Ok(film)
    }
});

struct Mutation;

graphql_object!(Mutation: Context |&self| {
    field create_film(&executor, input: CreateFilm) -> Result<Option<Film>, FieldError> {
        let db = executor.context().db_conn()?;
        let film = diesel::insert_into(films::table)
            .values(&input)
            .get_result(&db)
            .ok();

        Ok(film)
    }
});

#[derive(Debug, Extract, Deserialize)]
struct GraphQLRequest {
    #[serde(flatten)]
    gq: juniper::http::GraphQLRequest,
}

impl GraphQLRequest {
    pub fn execute<CtxT, QueryT, MutationT>(
        &self,
        root_node: &RootNode<'_, QueryT, MutationT>,
        context: &CtxT,
    ) -> GraphQLResponse
    where
        QueryT: GraphQLType<Context = CtxT>,
        MutationT: GraphQLType<Context = CtxT>,
    {
        let response = self.gq.execute(root_node, context);
        let status = if response.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        let json = serde_json::to_value(&response).unwrap();

        GraphQLResponse {
            inner: json,
            status: status.into(),
        }
    }
}

#[derive(Debug, Response, Serialize)]
struct GraphQLResponse {
    #[serde(flatten)]
    inner: serde_json::Value,

    #[web(header)]
    status: u16,
}

struct Api {
    schema: Schema,
    context: Context,
}

impl_web! {
    impl Api {
        #[get("/")]
        #[content_type("text/html; charset=utf-8")]
        async fn graphiql(&self) -> String {
            juniper::http::playground::playground_source("/graphql")
        }

        #[get("/graphql")]
        #[content_type("application/json")]
        async fn get_graphql_handler(&self, query_string: GraphQLRequest) -> GraphQLResponse {
            query_string.execute(&self.schema, &self.context)
        }

        #[post("/graphql")]
        #[content_type("application/json")]
        async fn post_graphql_handler(&self, body: GraphQLRequest) -> GraphQLResponse {
            body.execute(&self.schema, &self.context)
        }
    }
}

fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_conn_pool = r2d2::Pool::<ConnectionManager<PgConnection>>::new(manager).unwrap();

    let addr = "127.0.0.1:8080".parse().unwrap();
    println!("GraphQL API running on: {}", addr);

    ServiceBuilder::new()
        .resource(Api {
            schema: Schema::new(Query, Mutation),
            context: Context { db_conn_pool },
        })
        .run(&addr)
        .unwrap();
}
