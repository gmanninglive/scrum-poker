FROM node:18.16.0-slim as node_build
WORKDIR /scrum-poker

COPY . .

RUN yarn install
RUN yarn build

# Build Stage
FROM rust:1.69 as rust_build

# create a new empty shell project
RUN USER=root cargo new --bin scrum-poker
WORKDIR /scrum-poker

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

RUN rm target/release/deps/scrum_poker*

# copy your source tree
COPY ./src ./src
COPY ./templates ./templates

# build for release
RUN cargo build --release

FROM rust:1.69

COPY --from=rust_build /scrum-poker/target/release/scrum-poker /app/bin/scrum-poker
COPY --from=node_build /scrum-poker/assets /app/assets
COPY --from=node_build /scrum-poker/node_modules /app/node_modules

RUN chown -R $APP_USER:$APP_USER /app

USER $APP_USER
WORKDIR /app

EXPOSE 3000
ENV PORT 3000

CMD ["./bin/scrum-poker"]                                                                                                                                             0.1s 
