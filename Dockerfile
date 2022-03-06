FROM rust:latest

RUN apt-get update -y && apt-get install -y libudev-dev