# DDNS based on Cloudflare Worker

## Usage
1. Copy `wrangler_example.toml` as `wrangler.toml`, and set:
    - `ZONE_ID`: You can find it in your [domain homepage](https://dash.cloudflare.com/).
    - `EMAIL`: Your Cloudflare account email.
    - `KEY`: Your Cloudflare API key, generate it [here](https://dash.cloudflare.com/profile/api-tokens) and allow it modify dns record.
    - `TOKEN_ddns.example.com`: Your DDNS token, any random long string.

2. Run `wrangler publish`.

3. You can `curl -X POST https://ddns.YOUR_DOMAIN.workers.dev/update/ddns.example.com/MY_TOKEN` to update the record to the client ip address. If you want to update different subdomains, you may add multiple key-values like `TOKEN_ddns2.example.com`.

## Note
Since I have a complicated network topology at home, sometime the egress address is a foreign country IP. My dynamic IP address is only in China, so I wrote a fixed `CN` condition inside the code. You can delete it or modify it to your own needs.

Wow so wasm!