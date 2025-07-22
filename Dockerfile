FROM scratch AS base
WORKDIR /builder
COPY . .

FROM archlinux:base@sha256:7291088b2b8f143e29df31e80b254624c2a28d476a1820f88efcd8fb21e1360b
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
