FROM debian:buster

RUN apt-get update && apt-get install -y build-essential libtool autotools-dev automake pkg-config bsdmainutils python3 libevent-dev libboost-system-dev libboost-filesystem-dev libboost-test-dev libboost-thread-dev
RUN apt-get install git wget -y
RUN git clone https://github.com/namecoin/namecoin-core.git && cd namecoin-core \
    && ./contrib/install_db4.sh `pwd` \
    && export BDB_PREFIX='/namecoin-core/db4' \
    && ./autogen.sh \
    && ./configure BDB_LIBS="-L${BDB_PREFIX}/lib -ldb_cxx-4.8" BDB_CFLAGS="-I${BDB_PREFIX}/include" \
    && make \
    && make install

ADD namecoin.conf /root/.namecoin/

EXPOSE 18443

CMD ["namecoind", "-regtest", "-wallet=~/.namecoin/"]
