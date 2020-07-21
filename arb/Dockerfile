FROM alpine:latest

RUN apk update && apk add bitcoin bitcoin-cli

COPY bitcoin.conf /bitcoin.conf

EXPOSE 18332/tcp

ENTRYPOINT ["/usr/bin/bitcoind"]
CMD ["-conf=/bitcoin.conf", "-regtest", "-rest=1", "-server=1", "-printtoconsole", "-txindex=1"]
