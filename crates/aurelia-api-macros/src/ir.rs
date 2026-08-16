//! Intermediate Representation (IR) for API definitions
//!
//! This module defines the data structures that represent a parsed API trait.
//! These structures are the input for all code generators.

use proc_macro2::Ident;
use syn::Type;

/// A complete API definition parsed from a trait
pub struct ApiDefinition {
    pub methods: Vec<ApiMethod>,
}

/// A single API method
pub struct ApiMethod {
    pub name: Ident,
    pub http_method: HttpMethod,
    pub path: String,
    pub path_params: Vec<PathParam>,
    pub query_params: Vec<QueryParam>,
    pub body_param: Option<BodyParam>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// A path parameter extracted from the URL pattern
pub struct PathParam {
    pub name: Ident,
    pub ty: Type,
}

/// A query parameter (optional, not in path)
pub struct QueryParam {
    pub name: Ident,
    pub ty: Type,
    pub optional: bool,
}

/// The body parameter for POST/PUT/PATCH
pub struct BodyParam {
    pub name: Ident,
    pub ty: Type,
}
