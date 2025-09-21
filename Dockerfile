FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:8fe9851fcd8bd23ea0e6bfa99483d8778e40b45e0fd7bb8188c94cbdd8c25a38
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
