FROM paritytech/ci-linux:1.71.0-bullseye

WORKDIR /var/www/hashed-substrate

COPY . .
RUN cargo build --release
# RUN git clone https://github.com/hashed-io/hashed-substrate-parachain.git hashed-substrate && cd hashed-substrate && git checkout main && cargo build --release

# COPY ./target/release/hashed-parachain .
# COPY ./resources/* resources/.
# COPY ./scripts/start_collator.sh .

EXPOSE 30333 40333 9933 9944 9946

WORKDIR /var/www/hashed-substrate

CMD [ "/var/www/hashed-substrate/scripts/start_collator.sh" ]
