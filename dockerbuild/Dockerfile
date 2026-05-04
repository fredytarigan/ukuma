#
# First stage:
#    building application binary
#

FROM docker.io/rust:1.95 AS builder

# set our working directory 
WORKDIR /app

# copy source code to the container
COPY . .

# set required environment variables
ENV RUSTFLAGS='-C link-args=-s'

# build the server binary
RUN apk add --no-cache --update musl-dev curl \
    && cargo build --release

#
# Second stage:
#   create the final container with the server binary
#

FROM docker.io/alpine:3.23

LABEL maintainer="Fredy Samuel B Tarigan <fredy@tarigan.id>"
LABEL org.opencontainers.image.source="https://github.com/fredytarigan/ukuma"

# set our working directory
WORKDIR /app

# copy the server binary from the builder stage
COPY --from=builder /app/target/release/ukuma /app/ukuma
COPY dockerbuild/entrypoint.sh /app/entrypoint.sh

RUN apk add --update --no-cache \
    && apk add --no-cache ca-certificates tzdata curl bash \
    && addgroup -S app \
    && adduser -H -D -S app -G app \
    && chown -R app:app /app

USER app

# set the entrypoint
ENTRYPOINT ["./entrypoint.sh"]