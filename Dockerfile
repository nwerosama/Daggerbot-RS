FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:d2094076ea97044f0a0d7e8ea7ce025cc4fb9409b3bd5c4749c21728c204d490
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
