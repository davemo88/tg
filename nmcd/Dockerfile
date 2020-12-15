FROM debian:buster

RUN apt-get update && apt-get install -y build-essential libtool autotools-dev automake pkg-config bsdmainutils python3 libevent-dev libboost-system-dev libboost-filesystem-dev libboost-test-dev libboost-thread-dev
RUN apt-get install git -y
RUN git clone https://github.com/namecoin/namecoin-core.git && cd namecoin-core \
    && ./autogen.sh \
    && ./configure --disable-wallet \
    && make \
    && make install

ADD namecoin.conf ~/.namecoin/

EXPOSE 18443

CMD ["namecoind", "-regtest", "-wallet=~/.namecoin/"]
