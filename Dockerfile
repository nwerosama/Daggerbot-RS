FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:5519bb4afc78843352ea5afb3ea022c76c93b22f6c762c87cdc2b6885860965b
LABEL org.opencontainers.image.source="https://github.com/nwerosama/Daggerbot-RS"
ENV RUST_LOG=info
RUN pacman -Syu --noconfirm && \
  rm -rf /var/cache/pacman/pkg/** && \
  rm -rf /usr/share/{man,doc,info}
WORKDIR /daggerbot
COPY --from=base /builder/target/release/daggerbot .
COPY --from=base /builder/src/internals/assets/presence.toml .
COPY --from=base /builder/src/plugins/ plugins/
COPY --from=base /builder/schemas/ schemas/
CMD [ "./daggerbot" ]
