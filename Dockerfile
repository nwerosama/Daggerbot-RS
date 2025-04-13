FROM scratch AS base
WORKDIR /builder
COPY . .

FROM adelielinux/adelie:1.0-beta6
LABEL org.opencontainers.image.source="https://github.com/nwerosama/Daggerbot-RS"
RUN apk add --no-cache libgcc
WORKDIR /daggerbot
COPY --from=base /builder/target/x86_64-unknown-linux-musl/release/daggerbot .
COPY --from=base /builder/src/internals/assets/presence.toml .
COPY --from=base /builder/src/plugins/ plugins/
COPY --from=base /builder/schemas/ schemas/
CMD [ "./daggerbot" ]
