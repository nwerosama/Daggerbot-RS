FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:104d24b4464e89a16566d3e68ce0e2707aa15258c690ee9bef755930e8bc1c2d
LABEL org.opencontainers.image.source="https://github.com/nwerosama/Daggerbot-RS"
ENV RUST_LOG=info
RUN pacman -Syu --noconfirm && \
  rm -rf /var/cache/pacman/pkg/** && \
  rm -rf /usr/share/{man,doc,info}
WORKDIR /daggerbot
COPY --from=base /builder/target/release/daggerbot .
COPY --from=base /builder/assets/presence.toml .
COPY --from=base /builder/src/plugins/ plugins/
COPY --from=base /builder/schemas/ schemas/
EXPOSE 9000/tcp
CMD [ "./daggerbot" ]
