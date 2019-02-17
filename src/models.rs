// Copyright 2019 James Chapman

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::schema::films;

#[derive(Debug, GraphQLObject, Queryable)]
pub struct Film {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub title: String,
    pub release_year: i32,
    pub summary: String,
    pub runtime_mins: i32,
}

#[derive(Debug, GraphQLInputObject, Insertable)]
#[table_name = "films"]
pub struct CreateFilm {
    pub title: String,
    pub release_year: i32,
    pub summary: String,
    pub runtime_mins: i32,
}
