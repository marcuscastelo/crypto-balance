ADDRESS=$1
AUTH_TOKEN=$2
TURNSTILE_TOKEN=$3

curl "https://portfolio-api-public.sonar.watch/v1/portfolio/fetch?address=$ADDRESS&addressSystem=solana" \
  -H "accept: application/json, text/plain, */*" \
  -H "accept-language: en-GB,en;q=0.7" \
  -H "authorization: $AUTH_TOKEN" \
  -H "cache-control: no-cache" \
  -H "origin: https://sonar.watch" \
  -H "pragma: no-cache" \
  -H "priority: u=1, i" \
  -H "referer: https://sonar.watch/" \
  -H 'sec-ch-ua: "Not)A;Brand";v="99", "Brave";v="127", "Chromium";v="127"' \
  -H "sec-ch-ua-mobile: ?0" \
  -H 'sec-ch-ua-platform: "Linux"' \
  -H "sec-fetch-dest: empty" \
  -H "sec-fetch-mode: cors" \
  -H "sec-fetch-site: same-site" \
  -H "sec-gpc: 1" \
  -H "user-agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36" \
  -H "x-turnstile-token: $TURNSTILE_TOKEN"
