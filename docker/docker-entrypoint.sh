#!/bin/sh
# Translates SUPERDBA_* environment variables into `superdba web` flags.
#
#   docker run ... ghcr.io/superdba/superdba              -> superdba web <flags from env>
#   docker run ... ghcr.io/superdba/superdba --debug      -> extra flags are appended
#   docker run ... ghcr.io/superdba/superdba healthcheck  -> container health probe
#   docker run ... ghcr.io/superdba/superdba sh           -> any other command runs as is
set -eu

port="${SUPERDBA_PORT:-8080}"
public_url="${SUPERDBA_PUBLIC_URL:-http://localhost:${port}}"
public_url="${public_url%/}"

# The server only answers Host headers of its public URL, probes included.
public_host() {
    host="${public_url#*://}"
    printf '%s' "${host%%/*}"
}

is_true() {
    case "$1" in 1 | true | TRUE | True | yes | on) return 0 ;; *) return 1 ;; esac
}

# VAR_FILE -> VAR, the Docker/Kubernetes secrets convention.
file_env() {
    var="$1"
    eval "value=\${$var:-}"
    eval "file=\${${var}_FILE:-}"
    if [ -n "$value" ] && [ -n "$file" ]; then
        echo "superdba: set either $var or ${var}_FILE, not both" >&2
        exit 1
    fi
    if [ -n "$file" ]; then
        value="$(cat "$file")"
        export "$var=$value"
    fi
    unset "${var}_FILE"
}

if [ "${1:-}" = "healthcheck" ]; then
    exec curl -fsS --max-time 4 -o /dev/null \
        -H "Host: $(public_host)" "http://127.0.0.1:${port}/healthz"
fi

case "${1:-web}" in
    web) shift $(($# > 0 ? 1 : 0)) ;;
    -*) ;;
    *) exec "$@" ;;
esac

file_env SUPERDBA_WEB_PASSWORD
file_env SUPERDBA_WEB_PROXY_SECRET

auth="${SUPERDBA_AUTH:-password}"
case "$auth" in
    password)
        if [ -z "${SUPERDBA_WEB_PASSWORD:-}" ]; then
            password_file="${SUPERDBA_DATA_DIR:-/data}/web-password"
            if [ ! -s "$password_file" ]; then
                (umask 077 && head -c 18 /dev/urandom | base64 | tr '+/' '-_' >"$password_file")
                {
                    echo
                    echo "  No SUPERDBA_WEB_PASSWORD set. Generated login password:"
                    echo
                    echo "      $(cat "$password_file")"
                    echo
                    echo "  Stored in $password_file; set SUPERDBA_WEB_PASSWORD to choose your own."
                    echo
                } >&2
            else
                echo "superdba: using the generated login password from $password_file" >&2
            fi
            SUPERDBA_WEB_PASSWORD="$(cat "$password_file")"
            export SUPERDBA_WEB_PASSWORD
        fi
        ;;
    proxy)
        if [ -z "${SUPERDBA_WEB_PROXY_SECRET:-}" ]; then
            echo "superdba: SUPERDBA_AUTH=proxy requires SUPERDBA_WEB_PROXY_SECRET" >&2
            exit 1
        fi
        ;;
    *)
        echo "superdba: SUPERDBA_AUTH must be 'password' or 'proxy', got '$auth'" >&2
        exit 1
        ;;
esac

set -- web --no-open \
    --host "${SUPERDBA_HOST:-0.0.0.0}" \
    --port "$port" \
    --auth "$auth" \
    --public-url "$public_url" \
    --allowed-origin "$public_url" \
    "$@"

# Comma or whitespace separated; the public URL is always allowed.
for origin in $(printf '%s' "${SUPERDBA_ALLOWED_ORIGINS:-}" | tr ',' ' '); do
    origin="${origin%/}"
    [ "$origin" = "$public_url" ] || set -- "$@" --allowed-origin "$origin"
done
for proxy in $(printf '%s' "${SUPERDBA_TRUSTED_PROXIES:-}" | tr ',' ' '); do
    set -- "$@" --trusted-proxy "$proxy"
done
for root in $(printf '%s' "${SUPERDBA_SERVER_FILE_ROOTS:-}" | tr ',' ' '); do
    set -- "$@" --server-file-browser-root "$root"
done
if is_true "${SUPERDBA_ALLOW_HIGH_RISK:-false}"; then
    set -- "$@" --allow-high-risk
fi
if is_true "${SUPERDBA_DEBUG:-false}"; then
    set -- "$@" --debug
fi

echo "superdba: serving $public_url (auth: $auth)" >&2
exec superdba "$@"
