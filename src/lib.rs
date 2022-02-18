use worker::*;

mod cf;
mod cf_base;
mod utils;

const MAX_DOMAIN_LEN: usize = 64;

async fn set_record(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Get user token
    let user_token = match ctx.param("token") {
        Some(c) => c.to_string(),
        None => return Response::error("No token found", 403),
    };

    // Get user domain
    let user_domain = match ctx.param("domain") {
        Some(c) => c.to_string(),
        None => return Response::error("No domain found", 400),
    };

    // Validate user domain and token
    if user_domain
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && c != '.')
    {
        return Response::error("Domain contains invalid characters", 400);
    }
    if user_domain.len() > MAX_DOMAIN_LEN {
        return Response::error("Domain is too long", 400);
    }
    let token = match ctx.var(&format!("TOKEN_{}", user_domain)) {
        Ok(val) => val.to_string(),
        Err(_) => {
            return Response::error(
                format!("Token not found or invalid for domain {}", user_domain),
                403,
            )
        }
    };
    if token != user_token {
        return Response::error(
            format!("Token not found or invalid for domain {}", user_domain),
            403,
        );
    }

    // Load vars
    let zone_id = match ctx.var("ZONE_ID") {
        Ok(val) => val.to_string(),
        Err(_) => return Response::error("missing ZONE_ID", 500),
    };
    let email = match ctx.var("EMAIL") {
        Ok(val) => val.to_string(),
        Err(_) => return Response::error("missing EMAIL", 500),
    };
    let key = match ctx.var("KEY") {
        Ok(val) => val.to_string(),
        Err(_) => return Response::error("missing KEY", 500),
    };

    // We only serve for given country
    if let (Ok(country), Some(req_country)) = (ctx.var("COUNTRY"), req.cf().country()) {
        let cty = country.to_string();
        if !cty.is_empty() && cty != req_country {
            return Response::error(
                format!("Only available in {}, your country is {}", cty, req_country),
                403,
            );
        }
    }

    // Get user ip
    let user_ip = match req
        .headers()
        .get("cf-connecting-ip")
        .expect("internal error")
    {
        Some(user_ip) => user_ip,
        None => return Response::error("Missing cf-connecting-ip", 500),
    };
    let client = cf::Client::new(email, key);
    match client.update_dns(zone_id, user_domain, user_ip).await {
        Ok(_) => Response::ok("Update success"),
        Err(e) => Response::error(format!("Update failed: {}", e), 500),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::log_request(&req);
    utils::set_panic_hook();

    let router = Router::new();
    router
        .get("/", |_, _| Response::ok("DDNS Service by ihciah!"))
        .post("/", |_, _| Response::ok("DDNS Service by ihciah!"))
        .post_async("/update/:domain/:token", set_record)
        .run(req, env)
        .await
}
