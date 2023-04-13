FROM rustlang/rust:nightly as build
ENV pkg_config_allow_cross=1
WORKDIR /usr/src/tellme
COPY . .
RUN cargo install --path .
FROM gcr.io/distroless/cc-debian10
COPY --from=build /usr/local/cargo/bin/tellme /usr/local/bin/tellme
ENTRYPOINT [ "tellme" ]
