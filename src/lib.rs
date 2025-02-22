use worker::*;

mod cf;
mod cf_base;
mod utils;

const MAX_DOMAIN_LEN: usize = 64;

async fn set_record(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Get user token
    let Some(user_token) = ctx.param("token") else {
        return Response::error("No token found", 403);
    };

    // Get user domain
    let Some(user_domain) = ctx.param("domain") else {
        return Response::error("No domain found", 400);
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
    let Ok(token) = ctx.var(&format!("TOKEN_{user_domain}")) else {
        return Response::error(
            format!("Token not found or invalid for domain {user_domain}"),
            403,
        );
    };
    if !constant_time_eq::constant_time_eq(token.to_string().as_bytes(), user_token.as_bytes()) {
        return Response::error(
            format!("Token not found or invalid for domain {user_domain}"),
            403,
        );
    }

    // Load vars
    let Ok(zone_id) = ctx.var("ZONE_ID") else {
        return Response::error("missing ZONE_ID", 500);
    };
    let Ok(email) = ctx.var("EMAIL") else {
        return Response::error("missing EMAIL", 500);
    };
    let Ok(key) = ctx.var("KEY") else {
        return Response::error("missing KEY", 500);
    };

    // Get user ip
    let user_ip = if let Some(force_ip) = req.headers().get("force-ip").expect("internal error") {
        force_ip
    } else {
        // We only serve for given country if use cf-connecting-ip
        if let (Ok(country), Some(req_country)) =
            (ctx.var("COUNTRY"), req.cf().and_then(|f| f.country()))
        {
            let cty = country.to_string();
            if !cty.is_empty() && cty != req_country {
                return Response::error(
                    format!("Only available in {cty}, your country is {req_country}"),
                    403,
                );
            }
        }
        match req
            .headers()
            .get("cf-connecting-ip")
            .expect("internal error")
        {
            Some(ip) => ip,
            None => return Response::error("Missing cf-connecting-ip", 500),
        }
    };

    let client = cf::Client::new(email.to_string(), key.to_string());
    match client
        .update_dns(&zone_id.to_string(), user_domain, &user_ip)
        .await
    {
        Ok(_) => Response::ok("Update success"),
        Err(e) => Response::error(format!("Update failed: {e}"), 500),
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
