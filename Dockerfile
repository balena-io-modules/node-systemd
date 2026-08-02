# This dockerfile is not for production images, it has the following
# objectives
# - provide an example of install requirements
# - test the build of the project in a musl containerized environment
# - provide an install to run integration tests
# -	generate a binary to be published with the package to be used by node-pre-gyp
FROM alpine:3.20@sha256:d9e853e87e55526f6b2917df91a2115c36dd7c696a35be12163d44e6e2a4b6bc

RUN apk add --update --no-cache \
		build-base \
		nodejs~=20 \
		npm \
		dbus \
		rust cargo \
		curl

WORKDIR /usr/src/app

COPY package*.json ./
COPY Cargo.toml Cargo.lock tsconfig.json ./
COPY src ./src

# Install dependencies and build
# bindings
RUN npm install

# Create pre-gyp package
RUN npx node-pre-gyp package

COPY lib ./lib
COPY typings ./typings
COPY tests ./tests
COPY ./wait-for-it.sh /

CMD npm run test:integration
