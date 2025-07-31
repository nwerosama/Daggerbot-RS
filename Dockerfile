FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:5b1b7713182110aee0829fa9cc0571c451b391f0ad2bbafcf7a4ba907ee72ec3
LABEL org.opencontainers.image.source="https://github.com/nwerosama/Daggerbot-RS"
ENV RUST_LOG=info
# RUN pacman -Syu --noconfirm gdb strace && \
RUN pacman -Syu --noconfirm && \
  rm -rf /var/cache/pacman/pkg/** && \
  rm -rf /usr/share/{man,doc,info}
WORKDIR /daggerbot
COPY --from=base /builder/target/release/daggerbot .
COPY --from=base /builder/src/internals/assets/presence.toml .
COPY --from=base /builder/src/plugins/ plugins/
COPY --from=base /builder/schemas/ schemas/
EXPOSE 9000/tcp
CMD [ "./daggerbot" ]
# CMD ["gdb", "-return-child-result", "-batch", "-ex", "run", "-ex", "thread apply all bt", "-ex", "quit", "--args", "./daggerbot"]
