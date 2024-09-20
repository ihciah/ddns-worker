# DDNS based on Cloudflare Worker

## Usage
1. Copy `wrangler_example.toml` as `wrangler.toml`, and set:
    - `ZONE_ID`: You can find it in your [domain homepage](https://dash.cloudflare.com/).
    - `EMAIL`: Your Cloudflare account email.
    - `KEY`: Your Cloudflare API key, generate it [here](https://dash.cloudflare.com/profile/api-tokens) and allow it modify dns record.
    - `TOKEN_ddns.example.com`: The key format is `"TOKEN_" + your_sub_domain`. The value is the token for your sub-domain, you can set it as a long random string.

2. Run `npx wrangler deploy`.

3. You can `curl -X POST https://ddns.YOUR_WORKER_DOMAIN.workers.dev/update/ddns.example.com/EXAMPLE_TOKEN` to update the record to the client ip address. If you want to update different subdomains, you may add multiple key-values like `TOKEN_ddns2.example.com`.

4. If you want to specify ip address by your own, add the header `force-ip`. Note this can skip country checking.

## Note
You can also set `COUNTRY` variable in `wrangler.toml` to only allow requests from that country(for example, when you misused a VPN, the DNS record will be your VPN outbound IP, which maybe not what you want). If you don't want it, just leave the key blank. Country code format is as `ISO 3166-1 Alpha 2`, for example, `CN`, `US`, `SG`.

[The doc](https://developers.cloudflare.com/workers/languages/rust/) helps you prepare the environment.

Wow so wasm!
