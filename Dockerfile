FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:6e644b0d7ac1543ce6368d0cd9f919d6d234c718a721fdabd132f50acb0488b2
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
