FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:e5d672031f7479b0ce222486f1b5c8b07c931327d16050ad6078a8cd68cf870f
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
