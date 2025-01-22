FROM rust:1.84 as build

WORKDIR /usr/src/study_app_backend
COPY . .

RUN cargo install --path .

FROM alpine:latest

COPY --from=build /usr/local/cargo/bin/api-service /usr/local/bin/study_app_backend

CMD ["study_app_backend"]
