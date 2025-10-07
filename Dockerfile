FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:ca6af8049bd9dee3eb2bc3d620642ca1bc81b00f10aa08b12ee28ac56063be49
LABEL org.opencontainers.image.source="https://github.com/nwerosama/Daggerbot-RS"
ENV RUST_LOG=info
RUN pacman -Syu --noconfirm && \
  rm -rf /var/cache/pacman/pkg/** && \
  rm -rf /usr/share/{man,doc,info}
WORKDIR /daggerbot
COPY --from=base /builder/target/release/daggerbot .
COPY --from=base /builder/assets/presence.toml .
COPY --from=base /builder/schemas/ schemas/
EXPOSE 9000/tcp
CMD [ "./daggerbot" ]
