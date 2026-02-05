//! Parse syn types into our Intermediate Representation

use crate::ir::{ApiDefinition, ApiMethod, BodyParam, HttpMethod, PathParam, QueryParam};
use syn::{Attribute, FnArg, Ident, ItemTrait, Pat, TraitItem, TraitItemFn, Type, parse::Parse};

pub fn parse_api_trait(input: &ItemTrait) -> syn::Result<ApiDefinition> {
    let trait_name = input.ident.clone();
    let mut methods = Vec::new();

    for item in &input.items {
        if let TraitItem::Fn(method) = item {
            let api_method = parse_api_method(method)?;
            methods.push(api_method);
        }
    }

    Ok(ApiDefinition {
        trait_name,
        methods,
    })
}

fn parse_api_method(method: &TraitItemFn) -> syn::Result<ApiMethod> {
    let name = method.sig.ident.clone();

    // Find #[api(...)] attribute
    let api_attr = method
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("api"))
        .ok_or_else(|| syn::Error::new_spanned(&method.sig, "Missing #[api(...)] attribute"))?;

    let (http_method, path, desktop_only) = parse_api_attr(api_attr)?;

    // Parse function arguments
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();
    let mut body_param = None;

    for input in method.sig.inputs.iter() {
        match input {
            FnArg::Receiver(_) => {
                // Skip &self
                continue;
            }
            FnArg::Typed(pat_type) => {
                let param_name = extract_param_name(&pat_type.pat)?;
                let param_ty = (*pat_type.ty).clone();

                // Check if this param appears in the path pattern
                let path_pattern = format!("{{{}}}", param_name);
                if path.contains(&path_pattern) {
                    path_params.push(PathParam {
                        name: param_name,
                        ty: param_ty,
                    });
                } else if is_request_struct(&param_ty) {
                    // Body parameter (struct type)
                    body_param = Some(BodyParam {
                        name: param_name,
                        ty: param_ty,
                    });
                } else {
                    // Query parameter (primitive/optional)
                    let optional = is_option_type(&param_ty);
                    query_params.push(QueryParam {
                        name: param_name,
                        ty: param_ty,
                        optional,
                    });
                }
            }
        }
    }

    Ok(ApiMethod {
        name,
        http_method,
        path,
        path_params,
        query_params,
        body_param,
        return_type: method.sig.output.clone(),
        desktop_only,
    })
}

fn parse_api_attr(attr: &Attribute) -> syn::Result<(HttpMethod, String, bool)> {
    struct ApiAttr {
        method: HttpMethod,
        path: syn::LitStr,
        desktop_only: bool,
    }

    impl Parse for ApiAttr {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            // Parse HTTP method (GET, POST, etc.)
            let method_ident: Ident = input.parse()?;
            let method = match method_ident.to_string().as_str() {
                "GET" => HttpMethod::Get,
                "POST" => HttpMethod::Post,
                "PUT" => HttpMethod::Put,
                "DELETE" => HttpMethod::Delete,
                "PATCH" => HttpMethod::Patch,
                _ => {
                    return Err(syn::Error::new_spanned(
                        method_ident,
                        "Expected HTTP method (GET, POST, PUT, DELETE, PATCH)",
                    ));
                }
            };

            // Parse path string
            let path: syn::LitStr = input.parse()?;

            // Check for desktop_only
            let mut desktop_only = false;
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
                let flag: Ident = input.parse()?;
                if flag == "desktop_only" {
                    desktop_only = true;
                }
            }

            Ok(ApiAttr {
                method,
                path,
                desktop_only,
            })
        }
    }

    let parsed: ApiAttr = attr.parse_args()?;
    Ok((parsed.method, parsed.path.value(), parsed.desktop_only))
}

fn extract_param_name(pat: &Pat) -> syn::Result<Ident> {
    match pat {
        Pat::Ident(pat_ident) => Ok(pat_ident.ident.clone()),
        _ => Err(syn::Error::new_spanned(
            pat,
            "Expected simple identifier for parameter name",
        )),
    }
}

fn is_request_struct(ty: &Type) -> bool {
    // Check if type name ends with "Request" or is a complex struct
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let name = segment.ident.to_string();
        return name.ends_with("Request") || name.ends_with("Data");
    }
    false
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.first()
    {
        return segment.ident == "Option";
    }

    false
}

#[cfg(test)]
mod tests {
    use super::parse_api_trait;
    use crate::ir::HttpMethod;
    use syn::ItemTrait;

    #[test]
    fn parses_path_and_query_params() {
        let input: ItemTrait = syn::parse_str(
            r#"
            #[aurelia_api]
            pub trait Api {
                #[api(GET "/songs/{song_id}")]
                async fn get_song(&self, song_id: String, include: Option<String>) -> ApiResult<()>;
            }
            "#,
        )
        .expect("parse");

        let api = parse_api_trait(&input).expect("api");
        let method = &api.methods[0];
        assert_eq!(method.http_method, HttpMethod::Get);
        assert_eq!(method.path, "/songs/{song_id}");
        assert_eq!(method.path_params.len(), 1);
        assert_eq!(method.query_params.len(), 1);
        assert!(method.query_params[0].optional);
    }

    #[test]
    fn parses_desktop_only_flag() {
        let input: ItemTrait = syn::parse_str(
            r#"
            #[aurelia_api]
            pub trait Api {
                #[api(POST "/audio/play", desktop_only)]
                async fn audio_play(&self, url: String) -> ApiResult<()>;
            }
            "#,
        )
        .expect("parse");

        let api = parse_api_trait(&input).expect("api");
        let method = &api.methods[0];
        assert!(method.desktop_only);
    }

    #[test]
    fn missing_api_attribute_is_error() {
        let input: ItemTrait = syn::parse_str(
            r#"
            pub trait Api {
                async fn missing(&self) -> ApiResult<()>;
            }
            "#,
        )
        .expect("parse");

        let result = parse_api_trait(&input);
        assert!(result.is_err());
    }
}
