FROM rust:1.85.1-bookworm AS build
WORKDIR /src
COPY proto/ proto/
COPY services/route-engine/Cargo.toml services/route-engine/Cargo.lock* services/route-engine/build.rs services/route-engine/
COPY services/route-engine/src/ services/route-engine/src/
WORKDIR /src/services/route-engine
RUN cargo build --locked --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/services/route-engine/target/release/route-engine /route-engine
EXPOSE 50051
ENTRYPOINT ["/route-engine"]
