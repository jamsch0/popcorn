// Copyright 2019 James Chapman

#![feature(async_await, await_macro, futures_api)]
#![warn(clippy::all, rust_2018_idioms)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate tower_web;

use std::env;

use chrono::prelude::*;
use diesel::prelude::*;
use tokio::prelude::*;

use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use http::StatusCode;
use juniper::{EmptyMutation, GraphQLType, RootNode};
use juniper_from_schema::graphql_schema_from_file;
use tower_web::ServiceBuilder;
use uuid::Uuid;

mod models;
mod schema;

use crate::models::Film;
use crate::schema::films;

graphql_schema_from_file!("schema.graphql");

pub struct Context {
    db_conn_pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl Context {
    pub fn db_conn(&self) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
        self.db_conn_pool.get()
    }
}

impl juniper::Context for Context {}

type Schema = RootNode<'static, Query, EmptyMutation<Context>>;

struct Query;

impl QueryFields for Query {
    fn field_get_films(&self, executor: &juniper::Executor<'_, Context>, _: &QueryTrail<'_, Film, Walked>) -> juniper::FieldResult<Vec<Film>> {
        let db = executor.context().db_conn()?;

        let films = films::table.load(&db)?;
        Ok(films)
    }

    fn field_get_film(&self, executor: &juniper::Executor<'_, Context>, _: &QueryTrail<'_, Film, Walked>, id: Id) -> juniper::FieldResult<Option<Film>> {
        let id = Uuid::parse_str(&id.0)?;
        let db = executor.context().db_conn()?;

        let film = films::table.find(id).first(&db).ok();
        Ok(film)
    }
}

impl FilmFields for Film {
    fn field_id(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<Id> {
        Ok(Id::new(self.id.hyphenated().to_string()))
    }

    fn field_created_at(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&DateTime<Utc>> {
        Ok(&self.created_at)
    }

    fn field_updated_at(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&DateTime<Utc>> {
        Ok(&self.updated_at)
    }

    fn field_title(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&String> {
        Ok(&self.title)
    }

    fn field_release_year(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&i32> {
        Ok(&self.release_year)
    }

    fn field_summary(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&String> {
        Ok(&self.summary)
    }

    fn field_runtime_mins(&self, _: &juniper::Executor<'_, Context>) -> juniper::FieldResult<&i32> {
        Ok(&self.runtime_mins)
    }
}

#[derive(Debug, Extract, Deserialize)]
struct GraphQLRequest {
    #[serde(flatten)]
    gq: juniper::http::GraphQLRequest,
}

impl GraphQLRequest {
    pub fn execute<CtxT, QueryT, MutationT>(&self, root_node: &RootNode<'_, QueryT, MutationT>, context: &CtxT) -> GraphQLResponse
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
            juniper::http::graphiql::graphiql_source("/graphql")
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

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_conn_pool = r2d2::Pool::<ConnectionManager<PgConnection>>::new(manager).unwrap();

    let addr = "127.0.0.1:8080".parse().unwrap();
    println!("GraphQL API running on: {}", addr);

    ServiceBuilder::new()
        .resource(Api {
            schema: Schema::new(Query, EmptyMutation::<Context>::new()),
            context: Context { db_conn_pool },
        })
        .run(&addr)
        .unwrap();
}
