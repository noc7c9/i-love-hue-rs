_list:
    @just --list

watch CMD='check' +ARGS='':
    watchexec --watch src --restart --clear just {{CMD}} {{ARGS}}

check:
    cargo check

test:
    cargo test

build:
    cargo web deploy

build-release:
    cargo web deploy --release

start PORT='8000' HOST='0.0.0.0':
    cargo web start --auto-reload --port {{PORT}} --host {{HOST}}
